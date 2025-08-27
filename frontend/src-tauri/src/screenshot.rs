use base64::{engine::general_purpose, Engine as _};
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

#[tauri::command]
pub fn close_screenshot_overlay(app: AppHandle) -> Result<(), tauri::Error> {
    if let Some(window) = app.get_webview_window("screenshot_overlay") {
        window.close().unwrap();
    }

    Ok(())
}
