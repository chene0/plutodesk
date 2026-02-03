mod commands;
mod db;
mod dtos;
mod problem_naming;
mod screenshot;
mod session;
mod tray;

#[cfg(test)]
mod screenshot_session_test;

use commands::*;
use db::init_sqlite;
use screenshot::{
    close_screenshot_overlay, get_screenshot_data, receive_screenshot_data, take_screenshot,
    check_session_and_notify,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            close_screenshot_overlay,
            get_screenshot_data,
            receive_screenshot_data,
            // Folder commands
            create_folder,
            get_folder,
            get_folders_by_user,
            update_folder,
            delete_folder,
            // Course commands
            create_course,
            get_course,
            get_courses_by_folder,
            update_course,
            delete_course,
            // Set commands
            create_set,
            get_set,
            get_sets_by_course,
            update_set,
            delete_set,
            // Problem commands
            create_problem,
            get_problem,
            get_problems_by_set,
            update_problem,
            update_problem_stats,
            delete_problem,
            // Problem attempt commands
            create_problem_attempt,
            get_problem_attempt,
            get_attempts_by_problem,
            update_problem_attempt,
            delete_problem_attempt,
            // Session commands
            get_all_sessions,
            get_active_session,
            start_session,
            create_and_start_session,
            end_session,
            delete_session,
        ])
        .setup(|app| {
            // Enable logging in both debug and release builds
            // Default behavior: logs to stdout in debug, log directory in release
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;

            // Initialize screenshot data storage
            use screenshot::ScreenshotData;
            use std::sync::{Arc, Mutex};
            let app_handle = app.handle().clone();
            app_handle.manage::<ScreenshotData>(Arc::new(Mutex::new(None::<String>)));

            // Initialize session manager
            use session::{SessionManager, SessionManagerState};
            let sessions_path = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data directory: {}", e))?
                .join("sessions.json");
            
            let session_manager = SessionManager::load_from_file(&sessions_path)
                .unwrap_or_else(|e| {
                    log::warn!("Failed to load sessions from file: {}, creating new SessionManager", e);
                    SessionManager::new()
                });
            
            app_handle.manage::<SessionManagerState>(Arc::new(Mutex::new(session_manager)));

            // Initialize system tray
            #[cfg(desktop)]
            {
                tray::create_tray(&app_handle)?;
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let app_handle = app.handle().clone();

                // Initialize database
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = init_sqlite(&app_handle).await {
                        log::error!("Failed to initialize database: {}", e);
                    }
                });

                let screenshot_shortcut_ctrl =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
                let screenshot_shortcut_cmd =
                    Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyS);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            log::info!("{:?}", shortcut);
                            if shortcut == &screenshot_shortcut_ctrl
                                || shortcut == &screenshot_shortcut_cmd
                            {
                                let app_handle = _app.clone();
                                match event.state() {
                                    ShortcutState::Pressed => {
                                        tauri::async_runtime::spawn(async move {
                                            log::info!("Screenshot Shortcut Pressed!");
                                            // Check for active session first
                                            if check_session_and_notify(&app_handle).await {
                                                // Session is active, proceed with screenshot
                                                take_screenshot(app_handle).await.ok();
                                            }
                                            // If no session, notification was shown and user can click it
                                        });
                                    }
                                    ShortcutState::Released => {
                                        tauri::async_runtime::spawn(async move {
                                            log::info!("Screenshot Shortcut Released!");
                                        });
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(screenshot_shortcut_ctrl)?;
            }

            log::info!("Tauri application is running");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
