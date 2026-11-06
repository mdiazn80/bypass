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
    /// File variables, by key. A key lives in `vars` or here, never in both.
    #[serde(default)]
    pub files: BTreeMap<String, FileVar>,
}

/// A file stored (encrypted) in the vault. See `secrets::files`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileVar {
    pub filename: String,
    /// Base64, so binary files fit the JSON store and stay compact.
    pub content: String,
    pub size: u64,
    pub updated_at: String,
}

/// Public description of a file variable. The content is never part of it.
#[derive(Debug, Clone, Serialize)]
pub struct FileMeta {
    pub filename: String,
    pub size: u64,
    pub updated_at: String,
    /// Where the file is available while its context is active.
    pub path: String,
}

/// Root structure persisted (encrypted) in `store.enc`.
///
/// This is the only structure that ever touches the encrypted file. Secret
/// values live in `contexts[..].vars` and are never written in plaintext.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct VaultData {
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextEntry>,
}
