use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Emitter,
};

use crate::session::SessionManagerState;

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
                // Emit event to frontend to open session modal
                app.emit("open-session-modal", ()).ok();
            }
            "end_session" => {
                log::info!("End Session menu item clicked");
                // End the current session
                if let Some(session_manager) = app.try_state::<SessionManagerState>() {
                    let mut manager = session_manager.lock().unwrap();
                    manager.end_session();
                    
                    // Persist to file
                    if let Ok(app_data_dir) = app.path().app_data_dir() {
                        let sessions_path = app_data_dir.join("sessions.json");
                        if let Err(e) = manager.save_to_file(&sessions_path) {
                            log::error!("Failed to save sessions: {}", e);
                        }
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

