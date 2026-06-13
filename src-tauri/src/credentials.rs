use bypass_core::{BypassError, CredentialContext, Vault};
use tauri::State;

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

/// Returns metadata for all credential contexts.
#[tauri::command]
pub fn list_credential_contexts(state: State<AppState>) -> Result<Vec<CredentialContext>, String> {
    with_vault(&state, |v| v.list_contexts())
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

/// Deletes a credential context and all of its variables.
#[tauri::command]
pub fn delete_credential_context(state: State<AppState>, name: String) -> Result<(), String> {
    with_vault(&state, |v| v.delete_context(&name))
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

/// Creates or updates a variable in a context.
#[tauri::command]
pub fn set_credential_var(
    state: State<AppState>,
    context: String,
    key: String,
    value: String,
) -> Result<(), String> {
    with_vault(&state, |v| v.set_var(&context, &key, &value))
}

/// Removes a variable from a context.
#[tauri::command]
pub fn delete_credential_var(
    state: State<AppState>,
    context: String,
    key: String,
) -> Result<(), String> {
    with_vault(&state, |v| v.delete_var(&context, &key))
}
