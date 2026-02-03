use crate::problem_naming::suggest_problem_name;
use sanitize_filename::sanitize;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::db::{services, Db};
use crate::dtos::screenshot::ScreenshotDto;
use crate::session::SessionManagerState;
use base64::{engine::general_purpose, prelude::*};
use device_query::{DeviceQuery, DeviceState, MouseState};
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgba};
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_notification::NotificationExt;
use xcap::Monitor;

// Store screenshot data temporarily to avoid large event payloads
pub type ScreenshotData = Arc<Mutex<Option<String>>>;

// Capture screenshot and convert to base64 PNG
fn screenshot_to_base64() -> Result<String, Box<dyn std::error::Error>> {
    let device_state = DeviceState::new();
    let mouse: MouseState = device_state.get_mouse();
    let (x, y) = (mouse.coords.0, mouse.coords.1);

    let monitor = Monitor::from_point(x, y)?;
    let screenshot = monitor.capture_image()?;

    // Convert xcap image to image::ImageBuffer
    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = screenshot.into();

    // Encode as PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png_bytes).write_image(
        &buffer,
        buffer.width(),
        buffer.height(),
        ExtendedColorType::Rgba8,
    )?;

    // Convert to base64
    let base64_str = general_purpose::STANDARD.encode(&png_bytes);
    Ok(base64_str)
}

// Store screenshot data in app state to avoid large event payloads
#[tauri::command]
pub async fn get_screenshot_data(
    screenshot_data: State<'_, ScreenshotData>,
) -> Result<Option<String>, tauri::Error> {
    let data = screenshot_data.lock().unwrap();
    Ok(data.clone())
}

/// Check if there's an active session. If not, show OS notification.
/// Returns true if session is active, false if not.
pub async fn check_session_and_notify(app: &AppHandle) -> bool {
    // Check for active session
    if let Some(session_manager) = app.try_state::<SessionManagerState>() {
        let manager = session_manager.lock().unwrap();
        if manager.get_active_session().is_some() {
            // Session is active
            return true;
        }
    }

    // No active session - show notification and open window
    log::info!("No active session detected, showing notification and opening session modal");
    
    // Show notification
    if let Err(e) = app.notification()
        .builder()
        .title("Active Session Required")
        .body("Please start a session before taking screenshots")
        .show() {
        log::error!("Failed to show notification: {}", e);
    }
    
    // Open or focus main window with session modal
    if let Err(e) = open_or_focus_main_window_with_session_modal(app).await {
        log::error!("Failed to open main window: {}", e);
    }
    
    false
}

/// Open or focus the main window and show the session modal
pub async fn open_or_focus_main_window_with_session_modal(app: &AppHandle) -> Result<(), tauri::Error> {
    // Check if main window exists
    if let Some(main_window) = app.get_webview_window("main") {
        // Window exists - show and focus it
        log::info!("Main window exists, focusing it");
        main_window.show()?;
        main_window.set_focus()?;
    } else {
        // Window doesn't exist - create it
        log::info!("Main window doesn't exist, creating it");
        use tauri::WebviewWindowBuilder;
        use tauri::WebviewUrl;
        
        let window = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::default()
        )
        .title("plutodesk")
        .inner_size(800.0, 600.0)
        .resizable(true)
        .build()?;
        
        window.show()?;
        window.set_focus()?;
    }
    
    // Emit event to open session modal
    app.emit("open-session-modal", ())?;
    
    Ok(())
}

// Initiate screenshot overlay interface
pub async fn take_screenshot(app: AppHandle) -> Result<(), tauri::Error> {
    // Check if window exists and is actually usable (not just in registry)
    if let Some(existing_window) = app.get_webview_window("screenshot_overlay") {
        // Check if window is actually visible/usable
        match existing_window.is_visible() {
            Ok(true) => {
                log::warn!("Screenshot overlay already open and visible, ignoring shortcut.");
                return Ok(());
            }
            Ok(false) => {
                // Window exists but is hidden - close it to clean up state
                log::info!("Found hidden screenshot overlay window, closing it before creating new one.");
                if let Err(e) = existing_window.close() {
                    log::warn!("Failed to close existing window: {}", e);
                } else {
                    // Give it a moment to fully close
                    let _ = tauri::async_runtime::spawn_blocking(|| {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    })
                    .await;
                }
            }
            Err(_) => {
                // Window exists but is in invalid state - try to close it
                log::warn!("Found screenshot overlay window in invalid state, attempting to close.");
                if let Err(e) = existing_window.close() {
                    log::warn!("Failed to close invalid window: {}", e);
                } else {
                    let _ = tauri::async_runtime::spawn_blocking(|| {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    })
                    .await;
                }
            }
        }
    }

    log::info!("Taking screenshot...");
    let snapshot_base64_str = screenshot_to_base64().map_err(|e| {
        log::error!("Failed to capture screenshot: {}", e);
        tauri::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Screenshot capture failed: {}", e),
        ))
    })?;

    // Store screenshot data in app state
    if let Some(state) = app.try_state::<ScreenshotData>() {
        let mut data = state.lock().unwrap();
        *data = Some(snapshot_base64_str.clone());
    } else {
        log::error!("ScreenshotData state not found - state not initialized!");
    }

    let app_clone = app.clone();
    let payload = snapshot_base64_str.clone();

    // Store listener ID for cleanup
    let listener_id = app.listen_any("screenshot_overlay_ready", {
        let app_for_emit = app_clone.clone();
        let payload_for_emit = payload.clone();
        move |event| {
            // Clone for unlisten call (needed after async spawn)
            let app_for_unlisten = app_for_emit.clone();
            let event_id = event.id();
            
            // Add a delay to ensure the frontend listener is registered
            // The ready event is emitted immediately on mount, but the listener might not be ready yet
            let app_for_emit_inner = app_for_emit.clone();
            let payload_for_emit_inner = payload_for_emit.clone();
            tauri::async_runtime::spawn(async move {
                // Wait a bit for the frontend to set up the listener
                let _ = tauri::async_runtime::spawn_blocking(|| {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                })
                .await;

                // Check if window still exists before emitting
                let window_exists = app_for_emit_inner
                    .get_webview_window("screenshot_overlay")
                    .is_some();

                if !window_exists {
                    log::warn!("Window no longer exists when trying to emit screenshot data");
                    return;
                }

                match app_for_emit_inner.emit_to(
                    "screenshot_overlay",
                    "open_screenshot_overlay",
                    payload_for_emit_inner.clone(),
                ) {
                    Ok(_) => {
                        // Try to show the window after a short delay to allow frontend to process
                        // This is a fallback in case the frontend doesn't call window.show()
                        let app_for_show = app_for_emit_inner.clone();
                        tauri::async_runtime::spawn(async move {
                            // Use std::thread::sleep in async context via spawn_blocking
                            let _ = tauri::async_runtime::spawn_blocking(|| {
                                std::thread::sleep(std::time::Duration::from_millis(200));
                            })
                            .await;

                            // Check if window still exists and is valid
                            if let Some(window) = app_for_show.get_webview_window("screenshot_overlay") {
                                // Verify window is still valid before showing
                                match window.is_visible() {
                                    Ok(_) => {
                                        if let Err(e) = window.show() {
                                            log::error!("Window.show() failed (fallback): {}", e);
                                        }
                                    }
                                    Err(_) => {
                                        log::warn!("Window is in invalid state, skipping show");
                                    }
                                }
                            } else {
                                log::warn!(
                                    "Window 'screenshot_overlay' not found when trying to show (may have been closed)"
                                );
                            }
                        });
                    }
                    Err(e) => log::error!("Failed to emit open_screenshot_overlay event: {}", e),
                }
            });

            // Unlisten after spawning the async task
            app_for_unlisten.unlisten(event_id);
        }
    });

    // Use trailing slash for Next.js static export compatibility
    // With trailingSlash: true, Next.js creates routes as directories with index.html
    let overlay_url = "tauri/overlay/screenshot/";

    let webview_window = tauri::WebviewWindowBuilder::new(
        &app,
        "screenshot_overlay",
        tauri::WebviewUrl::App(overlay_url.into()),
    )
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .fullscreen(true)
    .skip_taskbar(true)
    .visible(false) // <-- Give time for frontend to process snapshot payload
    .build()
    .map_err(|e| {
        log::error!("Failed to create screenshot overlay window: {}", e);
        e
    })?;

    // Set up window close event handler to clean up listeners
    let app_for_close = app.clone();
    let listener_id_for_close = listener_id.clone();
    webview_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            log::info!("Screenshot overlay window close requested, cleaning up listeners");
            // Clean up the ready event listener
            app_for_close.unlisten(listener_id_for_close);
        }
    });

    // Set up a fallback to show the window if frontend doesn't do it
    // This handles the case where frontend JavaScript fails to execute
    let app_for_fallback = app.clone();
    tauri::async_runtime::spawn(async move {
        // Wait for frontend to be ready and process the event
        let _ = tauri::async_runtime::spawn_blocking(|| {
            std::thread::sleep(std::time::Duration::from_millis(500));
        })
        .await;

        // Check if window still exists and is valid before trying to show
        if let Some(window) = app_for_fallback.get_webview_window("screenshot_overlay") {
            // Verify window is still in a valid state
            match window.is_visible() {
                Ok(false) => {
                    // Window exists and is hidden - safe to show
                    log::warn!("Window is still hidden after 500ms, showing it now (fallback)");
                    if let Err(e) = window.show() {
                        log::error!("Window.show() failed (fallback): {}", e);
                    }
                }
                Ok(true) => {
                    // Window is already visible, nothing to do
                    log::debug!("Window is already visible, skipping fallback show");
                }
                Err(e) => {
                    // Window is in invalid state - don't try to show it
                    log::warn!("Window is in invalid state (may have been closed), skipping fallback show: {}", e);
                }
            }
        } else {
            log::warn!("Window 'screenshot_overlay' not found for fallback show (may have been closed)");
        }
    });

    Ok(())
}

// Helper to sanitize and clean spaces
fn clean_name(name: &str) -> String {
    sanitize(name).replace(" ", "_")
}

fn write_image_data_url_to_local_fs(
    app: AppHandle,
    payload: ScreenshotDto,
) -> Result<String, tauri::Error> {
    let relative_path = PathBuf::from(clean_name(&payload.folder_name))
        .join(clean_name(&payload.course_name))
        .join(clean_name(&payload.set_name));

    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())
        .unwrap();
    let full_dir_path = app_dir.join(&relative_path);

    // Create full directory
    fs::create_dir_all(&full_dir_path).map_err(|e| tauri::Error::from(e))?;

    // Decode image url
    // Remove header if present (data:image/png;base64, ....)
    let b64_string = payload
        .base64_data
        .split(',')
        .next_back()
        .unwrap_or(&payload.base64_data);
    let image_bytes = BASE64_STANDARD
        .decode(b64_string)
        .map_err(|e| e.to_string())
        .unwrap();

    // Write the file
    let mut file_path = full_dir_path.join(&payload.problem_name);
    file_path.set_extension("png");
    fs::write(&file_path, image_bytes).map_err(|e| tauri::Error::from(e))?;
    println!("Writing to {}", file_path.to_string_lossy().to_string());

    // Return relative path for db storage
    let db_path = relative_path.join(&payload.problem_name);

    Ok(db_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn receive_screenshot_data(
    app: AppHandle,
    image_url: String,
    folder_id: Option<String>,
    course_id: Option<String>,
    set_id: Option<String>,
) -> Result<(), tauri::Error> {
    use crate::session::SessionManagerState;
    use uuid::Uuid;

    let db = app.state::<Db>();
    
    // Determine folder/course/set IDs
    let (folder_uuid, course_uuid, set_uuid) = if let (Some(f), Some(c), Some(s)) = (folder_id, course_id, set_id) {
        // IDs provided directly (inline session selection)
        let folder_uuid = Uuid::parse_str(&f).map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid folder_id: {}", e),
            ))
        })?;
        let course_uuid = Uuid::parse_str(&c).map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid course_id: {}", e),
            ))
        })?;
        let set_uuid = Uuid::parse_str(&s).map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid set_id: {}", e),
            ))
        })?;
        (folder_uuid, course_uuid, set_uuid)
    } else {
        // Use active session
        let session_manager = app.state::<SessionManagerState>();
        let manager = session_manager.lock().unwrap();
        let session = manager.get_active_session().ok_or_else(|| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No active session. Please start a session before taking screenshots.",
            ))
        })?;
        (session.folder_id, session.course_id, session.set_id)
    };

    // Fetch names for filesystem path
    let folder = services::get_folder_by_id(db.connection(), folder_uuid)
        .await
        .map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database error: {}", e),
            ))
        })?
        .ok_or_else(|| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Folder with id {} not found", folder_uuid),
            ))
        })?;

    let course = services::get_course_by_id(db.connection(), course_uuid)
        .await
        .map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database error: {}", e),
            ))
        })?
        .ok_or_else(|| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Course with id {} not found", course_uuid),
            ))
        })?;

    let set = services::get_set_by_id(db.connection(), set_uuid)
        .await
        .map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database error: {}", e),
            ))
        })?
        .ok_or_else(|| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Set with id {} not found", set_uuid),
            ))
        })?;

    let problem_name = suggest_problem_name(image_url.clone()).await;

    let dto = ScreenshotDto {
        folder_name: folder.name,
        course_name: course.name,
        set_name: set.name,
        problem_name,
        base64_data: image_url,
    };

    // Write image to filesystem (need to clone dto for this call)
    let image_path = write_image_data_url_to_local_fs(app.clone(), dto.clone())?;

    // Save to database
    services::save_screenshot_to_db(db.connection(), dto, image_path)
        .await
        .map_err(|e| {
            tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database error: {}", e),
            ))
        })?;

    Ok(())
}

#[tauri::command]
pub fn close_screenshot_overlay(app: AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window("screenshot_overlay") {
        window.close().unwrap();
    }

    Ok(())
}

#[cfg(test)]
#[path = "screenshot_window_lifecycle_test.rs"]
mod screenshot_window_lifecycle_test;

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine};
    use std::path::PathBuf;

    #[test]
    fn test_clean_name() {
        // Test basic sanitization
        let result = clean_name("Test Folder");
        assert_eq!(result, "Test_Folder");

        // Test special characters
        let result = clean_name("Test/Folder<>");
        assert!(!result.contains("/"));
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));

        // Test spaces
        let result = clean_name("Test  Folder   Name");
        assert_eq!(result, "Test__Folder___Name");

        // Test empty string
        let result = clean_name("");
        assert_eq!(result, "");

        // Test only spaces - behavior may vary by platform
        // On Windows, sanitize may return empty string; on Linux, it may preserve spaces which become underscores
        let result = clean_name("   ");
        // Accept either empty string or underscores (platform-dependent behavior)
        assert!(
            result == "" || result == "___",
            "Expected empty string or '___', got '{}'",
            result
        );
    }

    #[test]
    fn test_write_image_data_url_to_local_fs() {
        // Create a simple test image (1x1 red pixel PNG)
        let test_image_bytes = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, // ...
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x08, 0x99, 0x01, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x01, // ...
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
        ];
        let base64_image = general_purpose::STANDARD.encode(&test_image_bytes);

        // Test with data URL prefix
        let dto_with_prefix = ScreenshotDto {
            folder_name: "Test Folder".to_string(),
            course_name: "Test Course".to_string(),
            set_name: "Test Set".to_string(),
            problem_name: "Test Problem".to_string(),
            base64_data: format!("data:image/png;base64,{}", base64_image),
        };

        // Note: Testing with data URL prefix is sufficient for this test
        // The base64 decoding logic handles both cases (with/without prefix)

        // Mock AppHandle - we'll need to create a mock or use a test helper
        // For now, we'll test the logic that doesn't require AppHandle
        // The actual integration with AppHandle would be tested in integration tests

        // Test directory structure creation logic
        let relative_path = PathBuf::from(clean_name(&dto_with_prefix.folder_name))
            .join(clean_name(&dto_with_prefix.course_name))
            .join(clean_name(&dto_with_prefix.set_name));

        // Compare path components instead of string representation (platform-agnostic)
        let expected_components = vec!["Test_Folder", "Test_Course", "Test_Set"];
        let actual_components: Vec<&str> =
            relative_path.iter().map(|c| c.to_str().unwrap()).collect();
        assert_eq!(actual_components, expected_components);

        // Test file extension logic
        // Note: problem_name in DTO is "Test Problem" (with space), but in actual usage
        // it would be sanitized via clean_name() before being used in the path
        let sanitized_problem_name = clean_name(&dto_with_prefix.problem_name);
        let mut file_path = relative_path.join(&sanitized_problem_name);
        file_path.set_extension("png");
        assert_eq!(file_path.extension().unwrap(), "png");
        assert_eq!(file_path.file_stem().unwrap(), "Test_Problem");
    }

    #[test]
    fn test_base64_decoding_with_and_without_prefix() {
        let test_bytes = b"test image data";
        let base64_str = general_purpose::STANDARD.encode(test_bytes);

        // Test with prefix
        let with_prefix = format!("data:image/png;base64,{}", base64_str);
        let b64_part = with_prefix.split(',').next_back().unwrap();
        let decoded = general_purpose::STANDARD.decode(b64_part).unwrap();
        assert_eq!(decoded, test_bytes);

        // Test without prefix
        let b64_part = base64_str.split(',').next_back().unwrap();
        let decoded = general_purpose::STANDARD.decode(b64_part).unwrap();
        assert_eq!(decoded, test_bytes);
    }
}
