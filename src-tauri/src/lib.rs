mod biometric;
mod commands;
mod hosts;
mod models;
mod state;
mod storage;
mod tray;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let contexts = storage::load_contexts();
    let config = storage::load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            contexts: Mutex::new(contexts),
            config: Mutex::new(config),
        })
        .setup(|app| {
            tray::create_tray(app.handle())?;

            let window = app.get_webview_window("main").unwrap();

            // Start minimized: hide window and dock icon, only show in menu bar
            let should_start_minimized = {
                let state = app.state::<AppState>();
                let config = state.config.lock().unwrap();
                config.start_minimized
            };
            if should_start_minimized {
                window.hide().ok();
                #[cfg(target_os = "macos")]
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Hide window on close if minimize_to_tray is enabled
            let handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let should_minimize = {
                        let state = handle.state::<AppState>();
                        let config = state.config.lock().unwrap();
                        config.minimize_to_tray
                    };
                    if should_minimize {
                        api.prevent_close();
                        if let Some(win) = handle.get_webview_window("main") {
                            win.hide().ok();
                        }
                        #[cfg(target_os = "macos")]
                        let _ = handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_contexts,
            commands::create_context,
            commands::update_context,
            commands::delete_context,
            commands::toggle_context,
            commands::get_hosts_content,
            commands::get_system_hosts,
            commands::get_config,
            commands::update_config,
            commands::apply_contexts,
            commands::check_biometric_available,
            commands::authenticate_biometric,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
