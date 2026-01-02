mod commands;
mod db;
mod dtos;
mod screenshot;

use commands::*;
use db::init_sqlite;
use screenshot::{
    close_screenshot_overlay, get_screenshot_data, receive_screenshot_data, take_screenshot,
};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
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
            // Subject commands
            create_subject,
            get_subject,
            get_subjects_by_course,
            update_subject,
            delete_subject,
            // Problem commands
            create_problem,
            get_problem,
            get_problems_by_subject,
            update_problem,
            update_problem_stats,
            delete_problem,
            // Problem attempt commands
            create_problem_attempt,
            get_problem_attempt,
            get_attempts_by_problem,
            update_problem_attempt,
            delete_problem_attempt,
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
                                            take_screenshot(app_handle).await.ok();
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
