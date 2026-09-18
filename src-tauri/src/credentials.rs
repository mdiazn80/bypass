use crate::secrets::{BypassError, CredentialContext, MergedVar, ResolvedVar, Vault};
use tauri::State;

use crate::agent;
use crate::state::AppState;

/// Runs `f` with a lazily-initialized vault, mapping core errors to strings so
/// the frontend can display them.
fn with_vault<T>(
    state: &State<AppState>,
    f: impl FnOnce(&Vault) -> Result<T, BypassError>,
) -> Result<T, String> {
    let mut guard = state.vault.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(Vault::new().map_err(|e| e.to_string())?);
    }
    let vault = guard.as_ref().expect("vault initialized above");
    f(vault).map_err(|e| e.to_string())
}

/// Returns metadata for all credential contexts in priority order (see
/// `AppConfig::credential_order`); contexts not yet ordered come last, by name.
#[tauri::command]
pub fn list_credential_contexts(state: State<AppState>) -> Result<Vec<CredentialContext>, String> {
    let mut contexts = with_vault(&state, |v| v.list_contexts())?;
    if let Ok(cfg) = state.config.lock() {
        contexts.sort_by(|a, b| {
            cfg.credential_rank(&a.name)
                .cmp(&cfg.credential_rank(&b.name))
                .then_with(|| a.name.cmp(&b.name))
        });
    }
    Ok(contexts)
}

/// Creates a new, empty credential context.
#[tauri::command]
pub fn create_credential_context(
    state: State<AppState>,
    name: String,
    description: String,
) -> Result<(), String> {
    with_vault(&state, |v| v.create_context(&name, &description))
}

/// Updates a credential context description.
#[tauri::command]
pub fn update_credential_context(
    state: State<AppState>,
    name: String,
    description: String,
) -> Result<(), String> {
    with_vault(&state, |v| v.update_context(&name, &description))
}

/// Renames a credential context. All variables are preserved under the new name.
/// Returns an error if `new_name` is already in use.
#[tauri::command(rename_all = "snake_case")]
pub fn rename_credential_context(
    state: State<AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err("Context name cannot be empty".to_string());
    }
    let result = with_vault(&state, |v| v.rename_context(&old_name, trimmed));
    if result.is_ok() {
        // Keep the active/priority lists in sync so they do not dangle.
        if let Ok(mut cfg) = state.config.lock() {
            cfg.rename_credential(&old_name, trimmed);
            let _ = crate::storage::save_config(&cfg);
        }
        agent::bump_gen(&state);
    }
    result
}

/// Deletes a credential context and all of its variables.
#[tauri::command]
pub fn delete_credential_context(state: State<AppState>, name: String) -> Result<(), String> {
    let result = with_vault(&state, |v| v.delete_context(&name));
    if result.is_ok() {
        // Drop the context from the active/priority lists.
        if let Ok(mut cfg) = state.config.lock() {
            cfg.forget_credential(&name);
            let _ = crate::storage::save_config(&cfg);
        }
        agent::bump_gen(&state);
    }
    result
}

/// Lists the variable keys of a context (values are not returned).
#[tauri::command]
pub fn list_credential_vars(state: State<AppState>, context: String) -> Result<Vec<String>, String> {
    with_vault(&state, |v| v.list_keys(&context))
}

/// Returns the decrypted value of a single variable. Only called when the user
/// explicitly reveals a value in the UI.
#[tauri::command]
pub fn get_credential_var(
    state: State<AppState>,
    context: String,
    key: String,
) -> Result<String, String> {
    with_vault(&state, |v| v.get_var(&context, &key))
}

/// Returns every variable of a context with its `{$VAR}` references resolved,
/// plus a description of any reference that could not be resolved. This is what
/// the editor previews; the stored templates still come from `get_credential_var`.
#[tauri::command]
pub fn resolve_credential_vars(
    state: State<AppState>,
    context: String,
) -> Result<Vec<ResolvedVar>, String> {
    with_vault(&state, |v| v.resolved_vars(&context))
}

/// The variables every shell receives: all active contexts layered by
/// priority, references resolved across them. This is the read-only summary
/// the UI shows, and the same set `agent.rs` serves.
#[tauri::command]
pub fn resolve_active_credential_vars(state: State<AppState>) -> Result<Vec<MergedVar>, String> {
    let active = state
        .config
        .lock()
        .map_err(|e| e.to_string())?
        .active_contexts_by_priority();
    with_vault(&state, |v| v.merged_vars(&active))
}

/// Creates or updates a variable in a context.
#[tauri::command]
pub fn set_credential_var(
    state: State<AppState>,
    context: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let result = with_vault(&state, |v| v.set_var(&context, &key, &value));
    if result.is_ok() {
        agent::bump_gen(&state);
    }
    result
}

/// Removes a variable from a context.
#[tauri::command]
pub fn delete_credential_var(
    state: State<AppState>,
    context: String,
    key: String,
) -> Result<(), String> {
    let result = with_vault(&state, |v| v.delete_var(&context, &key));
    if result.is_ok() {
        agent::bump_gen(&state);
    }
    result
}
