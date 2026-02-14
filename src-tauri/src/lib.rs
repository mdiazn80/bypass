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

            // Hide window on close if minimize_to_tray is enabled
            let handle = app.handle().clone();
            let window = app.get_webview_window("main").unwrap();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let state = handle.state::<AppState>();
                    let config = state.config.lock().unwrap();
                    if config.minimize_to_tray {
                        api.prevent_close();
                        if let Some(win) = handle.get_webview_window("main") {
                            win.hide().ok();
                        }
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
