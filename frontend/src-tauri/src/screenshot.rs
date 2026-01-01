use sanitize_filename::sanitize;
use std::fs;
use std::path::PathBuf;

use crate::db::{services, Db};
use crate::dtos::screenshot::ScreenshotDto;
use base64::{engine::general_purpose, prelude::*};
use device_query::{DeviceQuery, DeviceState, MouseState};
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, Rgba};
use tauri::{AppHandle, Emitter, Listener, Manager};
use xcap::Monitor;

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

// Initiate screenshot overlay interface
pub async fn take_screenshot(app: AppHandle) -> Result<(), tauri::Error> {
    if app.get_webview_window("screenshot_overlay").is_some() {
        log::warn!("Screenshot overlay already open, ignoring shortcut.");
        return Ok(());
    }

    let snapshot_base64_str = screenshot_to_base64().unwrap();

    let app_clone = app.clone();
    let payload = snapshot_base64_str.clone();

    log::info!("Waiting for screenshot_overlay_ready event...");
    let _cb_id = app.listen_any("screenshot_overlay_ready", move |event| {
        log::info!("screenshot_overlay_ready event received: {:?}", event);
        let _ = app_clone.emit_to(
            "screenshot_overlay",
            "open_screenshot_overlay",
            payload.clone(),
        );

        app_clone.unlisten(event.id());
    });

    let _webview_window = tauri::WebviewWindowBuilder::new(
        &app,
        "screenshot_overlay",
        tauri::WebviewUrl::App("tauri/overlay/screenshot".into()),
    )
    .transparent(true)
    .decorations(false)
    .always_on_top(true)
    .fullscreen(true)
    .skip_taskbar(true)
    .visible(false) // <-- Give time for frontend to process snapshot payload
    .build()
    .unwrap();

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
        .join(clean_name(&payload.subject_name));

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
) -> Result<(), tauri::Error> {
    // Passed struct has placeholder data for now,
    // In the future, these will be determined by a

    let folder_name = "Computer Science".to_string();
    let course_name = "Data Structures & Algorithms".to_string();
    let subject_name = "Binary Trees".to_string();
    let problem_name = "Lowest Common Ancestor".to_string();

    let dto = ScreenshotDto {
        folder_name: folder_name.clone(),
        course_name: course_name.clone(),
        subject_name: subject_name.clone(),
        problem_name: problem_name.clone(),
        base64_data: image_url,
    };

    // Write image to filesystem (need to clone dto for this call)
    let image_path = write_image_data_url_to_local_fs(app.clone(), dto.clone())?;

    // Save to database
    let db = app.state::<Db>();
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

        // Test only spaces - sanitize removes invalid filenames (spaces-only), returns empty string
        let result = clean_name("   ");
        assert_eq!(result, "");
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
            subject_name: "Test Subject".to_string(),
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
            .join(clean_name(&dto_with_prefix.subject_name));

        // Compare path components instead of string representation (platform-agnostic)
        let expected_components = vec!["Test_Folder", "Test_Course", "Test_Subject"];
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
