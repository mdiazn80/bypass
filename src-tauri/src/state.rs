use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use crate::agent::AgentHandle;
use crate::secrets::Vault;

use crate::models::{AppConfig, Context};

pub struct AppState {
    pub contexts: Mutex<Vec<Context>>,
    pub config: Mutex<AppConfig>,
    /// Lazily initialized credential vault. Kept as `Option` so a keychain
    /// failure does not prevent the rest of the app from starting.
    pub vault: Mutex<Option<Vault>>,
    /// Running shell agent (socket listener), if enabled.
    pub agent: Mutex<Option<AgentHandle>>,
    /// Bumped whenever the served environment changes (active context switch or
    /// a mutation of the active context's variables). Shells compare against
    /// their last-seen value to skip work when nothing changed.
    pub gen: AtomicU64,
}
