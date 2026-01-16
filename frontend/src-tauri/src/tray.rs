use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

use crate::session::SessionManagerState;
use tauri_plugin_notification::NotificationExt;

fn any_ui_window_visible<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.webview_windows()
        .values()
        .any(|w| w.is_visible().ok().unwrap_or(true))
}

fn focus_or_create_main_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // 1) Preferred: focus the main window by label.
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    // 2) Fallback: focus any existing window (label might differ).
    let windows = app.webview_windows();
    if let Some((_label, window)) = windows.iter().next() {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    // 3) No windows exist: create a main window.
    // Use a route-style URL so this works in both dev (Next dev server) and
    // production (static export where / maps to index.html).
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
        .title("plutodesk")
        .build()?;
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
    Ok(())
}

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let start_session_item = MenuItem::with_id(app, "start_session", "Start/Switch Session", true, None::<&str>)?;
    let end_session_item = MenuItem::with_id(app, "end_session", "End Session", true, None::<&str>)?;
    
    let menu = Menu::with_items(
        app,
        &[
            &start_session_item,
            &end_session_item,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                log::info!("Quit menu item clicked");
                app.exit(0);
            }
            "start_session" => {
                log::info!("Start/Switch Session menu item clicked");
                if let Err(e) = focus_or_create_main_window(app) {
                    log::warn!("Failed to focus/create main window from tray: {e}");
                }
                // Emit event to frontend to open session modal (after focusing)
                app.emit("open-session-modal", ()).ok();
            }
            "end_session" => {
                log::info!("End Session menu item clicked");
                // End the current session
                if let Some(session_manager) = app.try_state::<SessionManagerState>() {
                    let mut manager = session_manager.lock().unwrap();
                    let ended_name = manager.get_active_session().map(|s| s.name.clone());
                    manager.end_session();
                    
                    // Persist to file
                    if let Ok(app_data_dir) = app.path().app_data_dir() {
                        let sessions_path = app_data_dir.join("sessions.json");
                        if let Err(e) = manager.save_to_file(&sessions_path) {
                            log::error!("Failed to save sessions: {}", e);
                        }
                    }

                    // Notify user without stealing focus
                    let (title, body) = match ended_name {
                        Some(name) => ("Session Ended", format!("Ended: {name}")),
                        None => ("No Active Session", "There was no active session to end.".to_string()),
                    };
                    if let Err(e) = app.notification().builder().title(title).body(body).show() {
                        log::error!("Failed to show notification: {}", e);
                    }

                    // If UI is open/visible, notify frontend to refresh session state
                    if any_ui_window_visible(app) {
                        app.emit("session-state-changed", ()).ok();
                    }
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                log::info!("Tray icon left clicked");
            }
        })
        .build(app)?;

    Ok(())
}

// Note: Dynamic tray menu updates are not implemented yet
// The tray menu shows static items. Future enhancement could add
// dynamic "Active: [session name]" label that updates when session changes

