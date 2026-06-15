use tauri::AppHandle;

#[cfg(target_os = "macos")]
use smappservice_rs::{AppService, ServiceStatus, ServiceType};

#[cfg(not(target_os = "macos"))]
use tauri_plugin_autostart::ManagerExt;

/// Enables "launch at login".
#[tauri::command]
pub fn enable_autostart(app: AppHandle) -> Result<(), String> {
    enable(&app)
}

/// Disables "launch at login".
#[tauri::command]
pub fn disable_autostart(app: AppHandle) -> Result<(), String> {
    disable(&app)
}

/// Returns whether "launch at login" is currently enabled.
#[tauri::command]
pub fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    is_enabled(&app)
}

// On macOS, register the running app itself through SMAppService so System
// Settings shows "Bypass" with its icon instead of the signing team name.
#[cfg(target_os = "macos")]
fn enable(_app: &AppHandle) -> Result<(), String> {
    let service = AppService::new(ServiceType::MainApp);
    match service.register() {
        Ok(()) => Ok(()),
        // An already-registered service is a no-op for an idempotent toggle.
        Err(_) if service.status() == ServiceStatus::Enabled => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(target_os = "macos")]
fn disable(_app: &AppHandle) -> Result<(), String> {
    let service = AppService::new(ServiceType::MainApp);
    match service.unregister() {
        Ok(()) => Ok(()),
        // A service that is not registered is already disabled.
        Err(_) if service.status() == ServiceStatus::NotRegistered => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(target_os = "macos")]
fn is_enabled(_app: &AppHandle) -> Result<bool, String> {
    let service = AppService::new(ServiceType::MainApp);
    Ok(matches!(
        service.status(),
        ServiceStatus::Enabled | ServiceStatus::RequiresApproval
    ))
}

// Other platforms keep using tauri-plugin-autostart (LaunchAgent equivalents
// do not exist there and the plugin's behavior is already correct).
#[cfg(not(target_os = "macos"))]
fn enable(app: &AppHandle) -> Result<(), String> {
    app.autolaunch().enable().map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
fn disable(app: &AppHandle) -> Result<(), String> {
    app.autolaunch().disable().map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

/// Removes the stale `~/Library/LaunchAgents` plist written by the old
/// `tauri-plugin-autostart` LaunchAgent mode, which appeared in System Settings
/// under the signing team name instead of the app. If such a plist existed the
/// user had launch-at-login enabled, so re-arm that preference through
/// `SMAppService` to preserve it across the migration.
#[cfg(target_os = "macos")]
pub fn migrate_legacy_launch_agent(app: &AppHandle) {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    let dir = home.join("Library/LaunchAgents");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };

    let mut migrated = false;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("plist") {
            continue;
        }
        let is_ours = path.file_stem().and_then(|s| s.to_str()) == Some("Bypass")
            || std::fs::read_to_string(&path)
                .map(|c| c.contains("Bypass.app") || c.contains("com.mdiazn80.bypass"))
                .unwrap_or(false);
        if is_ours {
            let _ = std::fs::remove_file(&path);
            migrated = true;
        }
    }

    if migrated {
        let _ = enable(app);
    }
}
