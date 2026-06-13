use tauri::State;
use uuid::Uuid;

use crate::biometric;
use crate::hosts;
use crate::models::{AppConfig, Context};
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

/// Returns the current app configuration.
#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

/// Updates one or more app configuration fields and persists the result.
#[tauri::command]
pub fn update_config(
    state: State<AppState>,
    minimize_to_tray: Option<bool>,
    start_minimized: Option<bool>,
) -> Result<AppConfig, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if let Some(v) = minimize_to_tray {
        config.minimize_to_tray = v;
    }
    if let Some(v) = start_minimized {
        config.start_minimized = v;
    }
    let updated = config.clone();
    storage::save_config(&updated)?;
    Ok(updated)
}

/// Returns true if biometric authentication (Touch ID) is available on this device.
#[tauri::command]
pub fn check_biometric_available() -> bool {
    biometric::is_available()
}
