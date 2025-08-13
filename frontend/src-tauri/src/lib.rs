#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let screenshot_shortcut_ctrl =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
                let screenshot_shortcut_cmd =
                    Shortcut::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyS);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            println!("{:?}", shortcut);
                            if shortcut == &screenshot_shortcut_ctrl
                                || shortcut == &screenshot_shortcut_cmd
                            {
                                match event.state() {
                                    ShortcutState::Pressed => {
                                        println!("Screenshot Shortcut Pressed!");
                                        // Here you can add the logic to take a screenshot
                                    }
                                    ShortcutState::Released => {
                                        println!("Screenshot Shortcut Released!");
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(screenshot_shortcut_ctrl)?;
            }

            println!("Tauri application is running");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
