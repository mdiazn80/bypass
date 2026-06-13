use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use zeroize::Zeroizing;

use crate::crypto::{self, KEY_LEN};
use crate::error::BypassError;
use crate::keystore;
use crate::model::{ContextEntry, CredentialContext, VaultData};

/// Abstract, swappable storage surface for secret values.
///
/// Keeping this trait minimal means the underlying secret persistence
/// (currently [`HybridBackend`]) can be replaced without touching the rest of
/// the codebase.
pub trait SecretBackend: Send + Sync {
    fn get(&self, context: &str, key: &str) -> Result<String, BypassError>;
    fn set(&self, context: &str, key: &str, value: &str) -> Result<(), BypassError>;
    fn delete(&self, context: &str, key: &str) -> Result<(), BypassError>;
    fn list_keys(&self, context: &str) -> Result<Vec<String>, BypassError>;
    fn list_contexts(&self) -> Result<Vec<String>, BypassError>;
}

/// Hybrid backend: an OS-keychain-held master key encrypts an on-disk store
/// (`store.enc`) with ChaCha20-Poly1305. Secret values are never written in
/// plaintext and the master key never appears in the file.
pub struct HybridBackend {
    store_path: PathBuf,
    master_key: Zeroizing<[u8; KEY_LEN]>,
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Default location of the encrypted store, alongside the GUI's data.
pub fn default_store_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.mdiazn80.bypass")
        .join("store.enc")
}

impl HybridBackend {
    /// Builds a backend using the keychain-managed master key and the default
    /// store path. Falls back to `BYPASS_MASTER_KEY` when the OS keychain is
    /// unavailable.
    pub fn new() -> Result<Self, BypassError> {
        let master_key = match keystore::load_or_create_master_key() {
            Ok(key) => key,
            Err(BypassError::BackendUnavailable(msg)) => match keystore::master_key_from_env() {
                Some(result) => result?,
                None => return Err(BypassError::BackendUnavailable(msg)),
            },
            Err(e) => return Err(e),
        };
        Ok(Self {
            store_path: default_store_path(),
            master_key,
        })
    }

    /// Builds a backend with an explicitly provided master key. Used as a
    /// fallback when the OS keychain is unavailable.
    pub fn with_key(master_key: [u8; KEY_LEN]) -> Self {
        Self {
            store_path: default_store_path(),
            master_key: Zeroizing::new(master_key),
        }
    }

    /// Overrides the store path (primarily for tests and import/export).
    pub fn with_store_path(mut self, path: PathBuf) -> Self {
        self.store_path = path;
        self
    }

    pub(crate) fn load(&self) -> Result<VaultData, BypassError> {
        if !self.store_path.exists() {
            return Ok(VaultData::default());
        }
        let raw = fs::read(&self.store_path)?;
        if raw.is_empty() {
            return Ok(VaultData::default());
        }
        let plaintext = crypto::open(&self.master_key, &raw)?;
        let data = serde_json::from_slice(&plaintext)?;
        Ok(data)
    }

    pub(crate) fn store(&self, data: &VaultData) -> Result<(), BypassError> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let plaintext = Zeroizing::new(serde_json::to_vec(data)?);
        let ciphertext = crypto::seal(&self.master_key, &plaintext)?;
        // Write atomically so a crash never leaves a half-written store.
        let tmp = self.store_path.with_extension("enc.tmp");
        fs::write(&tmp, &ciphertext)?;
        fs::rename(&tmp, &self.store_path)?;
        Ok(())
    }

    /// Creates an empty context with the given description.
    pub fn create_context(&self, name: &str, description: &str) -> Result<(), BypassError> {
        let mut data = self.load()?;
        if data.contexts.contains_key(name) {
            return Err(BypassError::ContextExists(name.to_string()));
        }
        let ts = now();
        data.contexts.insert(
            name.to_string(),
            ContextEntry {
                description: description.to_string(),
                created_at: ts.clone(),
                updated_at: ts,
                vars: BTreeMap::new(),
            },
        );
        self.store(&data)
    }

    /// Deletes a context and all of its variables. Clears the global active
    /// pointer if it referenced the deleted context.
    pub fn delete_context(&self, name: &str) -> Result<(), BypassError> {
        let mut data = self.load()?;
        if data.contexts.remove(name).is_none() {
            return Err(BypassError::ContextNotFound(name.to_string()));
        }
        if data.active_context.as_deref() == Some(name) {
            data.active_context = None;
        }
        self.store(&data)
    }

    /// Updates a context description.
    pub fn set_description(&self, name: &str, description: &str) -> Result<(), BypassError> {
        let mut data = self.load()?;
        let entry = data
            .contexts
            .get_mut(name)
            .ok_or_else(|| BypassError::ContextNotFound(name.to_string()))?;
        entry.description = description.to_string();
        entry.updated_at = now();
        self.store(&data)
    }

    /// Returns public metadata for every context, sorted by name.
    pub fn context_meta(&self) -> Result<Vec<CredentialContext>, BypassError> {
        let data = self.load()?;
        Ok(data
            .contexts
            .into_iter()
            .map(|(name, entry)| CredentialContext {
                name,
                description: entry.description,
                created_at: entry.created_at,
                updated_at: entry.updated_at,
            })
            .collect())
    }

    /// Returns every variable of a context as a key/value map. Used to inject
    /// the environment for `run`/`shell`.
    pub fn vars(&self, context: &str) -> Result<BTreeMap<String, String>, BypassError> {
        let data = self.load()?;
        let entry = data
            .contexts
            .get(context)
            .ok_or_else(|| BypassError::ContextNotFound(context.to_string()))?;
        Ok(entry.vars.clone())
    }

    /// Returns the globally configured active context, if any.
    pub fn get_active(&self) -> Result<Option<String>, BypassError> {
        Ok(self.load()?.active_context)
    }

    /// Sets (or clears) the global active context. Validates that the named
    /// context exists.
    pub fn set_active(&self, name: Option<&str>) -> Result<(), BypassError> {
        let mut data = self.load()?;
        if let Some(n) = name {
            if !data.contexts.contains_key(n) {
                return Err(BypassError::ContextNotFound(n.to_string()));
            }
        }
        data.active_context = name.map(|s| s.to_string());
        self.store(&data)
    }

    /// Writes a passphrase-encrypted export of the whole vault to `path` for
    /// migration between machines. The OS keychain master key is not used.
    pub fn export_to(&self, path: &std::path::Path, passphrase: &str) -> Result<(), BypassError> {
        let data = self.load()?;
        let blob = crate::portable::export(&data, passphrase)?;
        fs::write(path, blob)?;
        Ok(())
    }

    /// Imports contexts from a passphrase-encrypted export, merging them into
    /// the current vault (existing contexts with the same name are replaced).
    pub fn import_from(
        &self,
        path: &std::path::Path,
        passphrase: &str,
    ) -> Result<(), BypassError> {
        let blob = fs::read(path)?;
        let imported = crate::portable::import(&blob, passphrase)?;
        let mut data = self.load()?;
        for (name, entry) in imported.contexts {
            data.contexts.insert(name, entry);
        }
        if data.active_context.is_none() {
            data.active_context = imported.active_context;
        }
        self.store(&data)
    }
}

impl SecretBackend for HybridBackend {
    fn get(&self, context: &str, key: &str) -> Result<String, BypassError> {
        let data = self.load()?;
        let entry = data
            .contexts
            .get(context)
            .ok_or_else(|| BypassError::ContextNotFound(context.to_string()))?;
        entry
            .vars
            .get(key)
            .cloned()
            .ok_or_else(|| BypassError::KeyNotFound(key.to_string(), context.to_string()))
    }

    fn set(&self, context: &str, key: &str, value: &str) -> Result<(), BypassError> {
        let mut data = self.load()?;
        let ts = now();
        let entry = data.contexts.entry(context.to_string()).or_insert_with(|| ContextEntry {
            description: String::new(),
            created_at: ts.clone(),
            updated_at: ts.clone(),
            vars: BTreeMap::new(),
        });
        entry.vars.insert(key.to_string(), value.to_string());
        entry.updated_at = ts;
        self.store(&data)
    }

    fn delete(&self, context: &str, key: &str) -> Result<(), BypassError> {
        let mut data = self.load()?;
        let entry = data
            .contexts
            .get_mut(context)
            .ok_or_else(|| BypassError::ContextNotFound(context.to_string()))?;
        if entry.vars.remove(key).is_none() {
            return Err(BypassError::KeyNotFound(key.to_string(), context.to_string()));
        }
        entry.updated_at = now();
        self.store(&data)
    }

    fn list_keys(&self, context: &str) -> Result<Vec<String>, BypassError> {
        let data = self.load()?;
        let entry = data
            .contexts
            .get(context)
            .ok_or_else(|| BypassError::ContextNotFound(context.to_string()))?;
        Ok(entry.vars.keys().cloned().collect())
    }

    fn list_contexts(&self) -> Result<Vec<String>, BypassError> {
        Ok(self.load()?.contexts.into_keys().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_backend(dir: &Path) -> HybridBackend {
        HybridBackend::with_key([7u8; KEY_LEN]).with_store_path(dir.join("store.enc"))
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bypass_core_test_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn set_get_roundtrip_and_encrypted_on_disk() {
        let dir = temp_dir("roundtrip");
        let backend = test_backend(&dir);
        backend.set("dev", "API_KEY", "secret123").unwrap();
        assert_eq!(backend.get("dev", "API_KEY").unwrap(), "secret123");

        let raw = fs::read(dir.join("store.enc")).unwrap();
        assert!(!raw.windows(9).any(|w| w == b"secret123"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_and_delete() {
        let dir = temp_dir("list");
        let backend = test_backend(&dir);
        backend.set("dev", "A", "1").unwrap();
        backend.set("dev", "B", "2").unwrap();
        assert_eq!(backend.list_keys("dev").unwrap(), vec!["A", "B"]);
        backend.delete("dev", "A").unwrap();
        assert_eq!(backend.list_keys("dev").unwrap(), vec!["B"]);
        assert_eq!(backend.list_contexts().unwrap(), vec!["dev"]);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn context_lifecycle_and_active() {
        let dir = temp_dir("lifecycle");
        let backend = test_backend(&dir);
        backend.create_context("prod", "Production").unwrap();
        assert!(backend.create_context("prod", "dup").is_err());
        backend.set_active(Some("prod")).unwrap();
        assert_eq!(backend.get_active().unwrap().as_deref(), Some("prod"));
        backend.delete_context("prod").unwrap();
        assert_eq!(backend.get_active().unwrap(), None);
        assert!(backend.set_active(Some("missing")).is_err());
        fs::remove_dir_all(&dir).ok();
    }
}
