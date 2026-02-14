use tauri::State;
use uuid::Uuid;

use crate::biometric;
use crate::hosts;
use crate::models::{AppConfig, Context};
use crate::state::AppState;
use crate::storage;

#[tauri::command]
pub fn list_contexts(state: State<AppState>) -> Vec<Context> {
    state.contexts.lock().unwrap().clone()
}

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

    let mut contexts = state.contexts.lock().unwrap();
    contexts.push(ctx.clone());
    storage::save_contexts(&contexts)?;
    Ok(ctx)
}

#[tauri::command]
pub fn update_context(
    state: State<AppState>,
    id: String,
    name: Option<String>,
    content: Option<String>,
) -> Result<Context, String> {
    let mut contexts = state.contexts.lock().unwrap();
    let ctx = contexts
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Context not found".to_string())?;

    if let Some(n) = name {
        ctx.name = n;
    }
    if let Some(c) = content {
        ctx.content = c;
    }
    ctx.updated_at = chrono::Utc::now().to_rfc3339();

    let updated = ctx.clone();
    storage::save_contexts(&contexts)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_context(state: State<AppState>, id: String) -> Result<(), String> {
    let mut contexts = state.contexts.lock().unwrap();
    let was_enabled = contexts.iter().find(|c| c.id == id).map(|c| c.enabled).unwrap_or(false);
    contexts.retain(|c| c.id != id);
    storage::save_contexts(&contexts)?;

    if was_enabled {
        hosts::apply_to_hosts(&contexts)?;
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_context(state: State<AppState>, id: String) -> Result<Context, String> {
    let mut contexts = state.contexts.lock().unwrap();
    let ctx = contexts
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or_else(|| "Context not found".to_string())?;

    ctx.enabled = !ctx.enabled;
    ctx.updated_at = chrono::Utc::now().to_rfc3339();

    let updated = ctx.clone();
    storage::save_contexts(&contexts)?;
    hosts::apply_to_hosts(&contexts)?;
    Ok(updated)
}

#[tauri::command]
pub fn get_hosts_content(state: State<AppState>) -> String {
    let contexts = state.contexts.lock().unwrap();
    hosts::build_managed_block(&contexts)
}

#[tauri::command]
pub fn get_system_hosts() -> Result<String, String> {
    hosts::read_system_hosts()
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(
    state: State<AppState>,
    minimize_to_tray: Option<bool>,
    start_minimized: Option<bool>,
) -> Result<AppConfig, String> {
    let mut config = state.config.lock().unwrap();
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

#[tauri::command]
pub fn apply_contexts(state: State<AppState>) -> Result<(), String> {
    let contexts = state.contexts.lock().unwrap();
    hosts::apply_to_hosts(&contexts)
}

#[tauri::command]
pub fn check_biometric_available() -> bool {
    biometric::is_available()
}

#[tauri::command]
pub fn authenticate_biometric() -> Result<(), String> {
    biometric::authenticate("Bypass needs to modify /etc/hosts")
}
