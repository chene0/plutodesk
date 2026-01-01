use sanitize_filename::sanitize;
use std::fs;
use std::path::PathBuf;

use crate::dtos::screenshot::ScreenshotDto;
use base64::{engine::general_purpose, prelude::*, Engine as _};
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
    fs::create_dir_all(&full_dir_path).map_err(|e| e.to_string());

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
    fs::write(&file_path, image_bytes).map_err(|e| e.to_string());
    println!("Writing to {}", file_path.to_string_lossy().to_string());

    // Return relative path for db storage
    let db_path = relative_path.join(&payload.problem_name);

    Ok(db_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn receive_screenshot_data(app: AppHandle, image_url: String) -> Result<(), tauri::Error> {
    // Passed struct has placeholder data for now,
    // In the future, these will be determined by a
    write_image_data_url_to_local_fs(
        app,
        ScreenshotDto {
            folder_name: "Computer Science".to_string(),
            course_name: "Data Structures & Algorithms".to_string(),
            subject_name: "Binary Trees".to_string(),
            problem_name: "Lowest Common Ancestor".to_string(),
            base64_data: image_url,
        },
    )?;

    Ok(())
}

#[tauri::command]
pub fn close_screenshot_overlay(app: AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window("screenshot_overlay") {
        window.close().unwrap();
    }

    Ok(())
}
