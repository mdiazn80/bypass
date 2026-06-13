use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Public, non-sensitive metadata describing a credential context.
///
/// Variable values are never part of this struct; they are only ever exposed
/// through explicit value getters so they are not accidentally serialized or
/// logged alongside metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialContext {
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Internal per-context record stored (encrypted) on disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ContextEntry {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
}

/// Root structure persisted (encrypted) in `store.enc`.
///
/// This is the only structure that ever touches the encrypted file. Secret
/// values live in `contexts[..].vars` and are never written in plaintext.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct VaultData {
    #[serde(default)]
    pub active_context: Option<String>,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextEntry>,
}
