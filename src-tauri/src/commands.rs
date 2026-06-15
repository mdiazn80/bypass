use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::agent;
use crate::biometric;
use crate::hosts;
use crate::models::{AppConfig, Context};
use crate::shell_install::{self, ShellStatus};
use crate::state::AppState;
use crate::storage;

/// Returns the full list of contexts stored in memory.
#[tauri::command]
pub fn list_contexts(state: State<AppState>) -> Vec<Context> {
    state.contexts.lock().unwrap().clone()
}

/// Creates a new empty context and persists it.
#[tauri::command]
pub fn create_context(state: State<AppState>, name: String) -> Result<Context, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let ctx = Context {
        id: Uuid::new_v4().to_string(),
        name,
        content: String::new(),
        enabled: false,
        created_at: now.clone(),
        updated_at: now,
    };

    let mut contexts = state.contexts.lock().map_err(|e| e.to_string())?;
    contexts.push(ctx.clone());
    storage::save_contexts(&contexts)?;
    Ok(ctx)
}

/// Updates a context's name and/or content.
/// If the context is currently enabled, re-applies hosts immediately so the
/// system file stays in sync with the new content.
#[tauri::command]
pub fn update_context(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    content: Option<String>,
) -> Result<Context, String> {
    let mut contexts = state.contexts.lock().map_err(|e| e.to_string())?;
    let ctx = contexts
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Context not found".to_string())?;

    let was_enabled = ctx.enabled;

    if let Some(n) = name {
        ctx.name = n;
    }
    if let Some(c) = content {
        ctx.content = c;
    }
    ctx.updated_at = chrono::Utc::now().to_rfc3339();

    let updated = ctx.clone();

    // Re-apply hosts before persisting so the system file is never stale.
    // If the write fails, the in-memory state is already updated but nothing
    // is persisted, so the user can retry without data loss.
    if was_enabled {
        hosts::apply_to_hosts(&contexts)?;
    }
    storage::save_contexts(&contexts)?;
    Ok(updated)
}

/// Deletes a context. If it was enabled, applies hosts first to remove its
/// entries before removing the context from disk.
#[tauri::command]
pub fn delete_context(state: State<AppState>, id: String) -> Result<(), String> {
    let mut contexts = state.contexts.lock().map_err(|e| e.to_string())?;
    let was_enabled = contexts
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.enabled)
        .unwrap_or(false);

    if was_enabled {
        // Build the future state without the deleted context and apply it first.
        // This way hosts is updated before we commit the deletion to disk.
        let future: Vec<_> = contexts.iter().filter(|c| c.id != id).cloned().collect();
        hosts::apply_to_hosts(&future)?;
    }

    contexts.retain(|c| c.id != id);
    storage::save_contexts(&contexts)?;
    Ok(())
}

/// Toggles a context on or off. Applies the new hosts state before persisting
/// so that a failed write leaves disk and hosts in a consistent state.
#[tauri::command]
pub fn toggle_context(state: State<AppState>, id: String) -> Result<Context, String> {
    let mut contexts = state.contexts.lock().map_err(|e| e.to_string())?;
    let ctx = contexts
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Context not found".to_string())?;

    ctx.enabled = !ctx.enabled;
    ctx.updated_at = chrono::Utc::now().to_rfc3339();

    let updated = ctx.clone();

    // Apply hosts first. If it fails, revert the toggle so in-memory state
    // stays consistent with what the system file actually contains.
    if let Err(e) = hosts::apply_to_hosts(&contexts) {
        if let Some(ctx) = contexts.iter_mut().find(|c| c.id == id) {
            ctx.enabled = !ctx.enabled;
        }
        return Err(e);
    }
    storage::save_contexts(&contexts)?;
    Ok(updated)
}

/// Returns the current system hosts file content.
#[tauri::command]
pub fn get_system_hosts() -> Result<String, String> {
    hosts::read_system_hosts()
}

/// Reads an arbitrary text file by absolute path. Used to import a context from
/// a file dropped onto the window (drag-and-drop paths are outside the fs scope).
#[tauri::command]
pub fn read_file_text(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {e}"))
}

/// Returns the current app configuration.
#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

/// Updates app configuration fields and persists the result.
#[tauri::command(rename_all = "snake_case")]
pub fn update_config(
    state: State<AppState>,
    minimize_to_tray: bool,
    start_minimized: bool,
) -> Result<AppConfig, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.minimize_to_tray = minimize_to_tray;
    config.start_minimized = start_minimized;
    let updated = config.clone();
    storage::save_config(&updated)?;
    Ok(updated)
}

/// Returns true if biometric authentication (Touch ID) is available on this device.
#[tauri::command]
pub fn check_biometric_available() -> bool {
    biometric::is_available()
}

// --- Shell integration ------------------------------------------------------

/// Builds the current shell-integration status snapshot for the UI.
fn build_status(state: &State<AppState>) -> ShellStatus {
    let (enabled, installed, active) = state
        .config
        .lock()
        .ok()
        .map(|c| {
            (
                c.shell_integration_enabled,
                c.shell_integration_installed,
                c.active_context.clone(),
            )
        })
        .unwrap_or((false, false, None));
    let socket_active = state.agent.lock().map(|a| a.is_some()).unwrap_or(false);
    ShellStatus {
        enabled,
        installed,
        socket_active,
        active_context: active,
        detected_shell: shell_install::detected_shell_label(),
        rc_path: shell_install::rc_path_string(),
    }
}

#[tauri::command]
pub fn get_shell_status(state: State<AppState>) -> ShellStatus {
    build_status(&state)
}

/// Sets which credential context's variables are served to shells.
#[tauri::command]
pub fn set_active_context(
    state: State<AppState>,
    name: Option<String>,
) -> Result<ShellStatus, String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.active_context = name;
        storage::save_config(&cfg)?;
    }
    agent::bump_gen(&state);
    Ok(build_status(&state))
}

/// Starts or stops the local shell agent (socket listener).
#[tauri::command]
pub fn set_shell_agent_enabled(
    app: AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<ShellStatus, String> {
    {
        let mut guard = state.agent.lock().map_err(|e| e.to_string())?;
        if enabled {
            if guard.is_none() {
                let handle = agent::start(app.clone()).map_err(|e| e.to_string())?;
                *guard = Some(handle);
            }
        } else if let Some(handle) = guard.take() {
            handle.stop();
        }
    }
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.shell_integration_enabled = enabled;
        storage::save_config(&cfg)?;
    }
    Ok(build_status(&state))
}

/// Writes the prompt hook into the user's shell startup file.
#[tauri::command]
pub fn install_shell_integration(
    app: AppHandle,
    state: State<AppState>,
) -> Result<ShellStatus, String> {
    shell_install::install(&app)?;
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.shell_integration_installed = true;
        storage::save_config(&cfg)?;
    }
    Ok(build_status(&state))
}

/// Removes the prompt hook from the user's shell startup file.
#[tauri::command]
pub fn uninstall_shell_integration(state: State<AppState>) -> Result<ShellStatus, String> {
    shell_install::uninstall()?;
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.shell_integration_installed = false;
        storage::save_config(&cfg)?;
    }
    Ok(build_status(&state))
}
