mod agent;
mod autostart;
mod biometric;
mod commands;
mod credentials;
mod hosts;
mod models;
mod secrets;
mod shell_install;
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
            vault: Mutex::new(None),
            agent: Mutex::new(None),
            // Start at 1 so a client with no prior generation (0) always syncs.
            gen: std::sync::atomic::AtomicU64::new(1),
        })
        .setup(|app| {
            tray::create_tray(app.handle())?;

            // Migrate away from the legacy LaunchAgent (shown under the signing
            // team name) to SMAppService-based login items.
            #[cfg(target_os = "macos")]
            autostart::migrate_legacy_launch_agent(app.handle());

            // Start the shell agent if it was enabled in a previous session.
            let shell_enabled = {
                let state = app.state::<AppState>();
                let config = state.config.lock().unwrap();
                config.shell_integration_enabled
            };
            if shell_enabled {
                match agent::start(app.handle().clone()) {
                    Ok(handle) => {
                        let state = app.state::<AppState>();
                        *state.agent.lock().unwrap() = Some(handle);
                    }
                    Err(e) => eprintln!("failed to start shell agent: {e}"),
                }
            }

            let window = app.get_webview_window("main")
                .ok_or("main window not found")?;

            // The window starts hidden (visible: false in tauri.conf.json).
            // Show it now unless the user wants to start minimized to tray.
            let should_start_minimized = {
                let state = app.state::<AppState>();
                let config = state.config.lock().unwrap();
                config.start_minimized
            };
            if should_start_minimized {
                #[cfg(target_os = "macos")]
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            } else {
                window.show().ok();
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
            commands::get_system_hosts,
            commands::read_file_text,
            commands::get_config,
            commands::update_config,
            autostart::enable_autostart,
            autostart::disable_autostart,
            autostart::is_autostart_enabled,
            commands::check_biometric_available,
            commands::get_shell_status,
            commands::set_active_context,
            commands::set_shell_agent_enabled,
            commands::install_shell_integration,
            commands::uninstall_shell_integration,
            credentials::list_credential_contexts,
            credentials::create_credential_context,
            credentials::update_credential_context,
            credentials::rename_credential_context,
            credentials::delete_credential_context,
            credentials::list_credential_vars,
            credentials::get_credential_var,
            credentials::set_credential_var,
            credentials::delete_credential_var,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
