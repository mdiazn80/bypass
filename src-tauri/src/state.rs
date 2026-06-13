use std::sync::Mutex;

use bypass_core::Vault;

use crate::models::{AppConfig, Context};

pub struct AppState {
    pub contexts: Mutex<Vec<Context>>,
    pub config: Mutex<AppConfig>,
    /// Lazily initialized credential vault. Kept as `Option` so a keychain
    /// failure does not prevent the rest of the app from starting.
    pub vault: Mutex<Option<Vault>>,
}
