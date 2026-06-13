use super::backend::{HybridBackend, SecretBackend};
use super::error::BypassError;
use super::model::CredentialContext;

/// High-level entry point used by the GUI.
///
/// Wraps a [`HybridBackend`] and adds context metadata management on top of the
/// raw secret CRUD.
pub struct Vault {
    backend: HybridBackend,
}

impl Vault {
    /// Builds a vault backed by the keychain-managed master key.
    pub fn new() -> Result<Self, BypassError> {
        Ok(Self {
            backend: HybridBackend::new()?,
        })
    }

    /// Builds a vault around an already-constructed backend (tests only).
    #[cfg(test)]
    pub fn with_backend(backend: HybridBackend) -> Self {
        Self { backend }
    }

    // --- Context metadata ---------------------------------------------------

    pub fn list_contexts(&self) -> Result<Vec<CredentialContext>, BypassError> {
        self.backend.context_meta()
    }

    pub fn create_context(&self, name: &str, description: &str) -> Result<(), BypassError> {
        self.backend.create_context(name, description)
    }

    pub fn update_context(&self, name: &str, description: &str) -> Result<(), BypassError> {
        self.backend.set_description(name, description)
    }

    pub fn delete_context(&self, name: &str) -> Result<(), BypassError> {
        self.backend.delete_context(name)
    }

    pub fn rename_context(&self, old_name: &str, new_name: &str) -> Result<(), BypassError> {
        self.backend.rename_context(old_name, new_name)
    }

    // --- Variables ----------------------------------------------------------

    pub fn list_keys(&self, context: &str) -> Result<Vec<String>, BypassError> {
        self.backend.list_keys(context)
    }

    pub fn get_var(&self, context: &str, key: &str) -> Result<String, BypassError> {
        self.backend.get(context, key)
    }

    pub fn set_var(&self, context: &str, key: &str, value: &str) -> Result<(), BypassError> {
        self.backend.set(context, key, value)
    }

    pub fn delete_var(&self, context: &str, key: &str) -> Result<(), BypassError> {
        self.backend.delete(context, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::crypto::KEY_LEN;
    use std::fs;

    fn vault(dir: &std::path::Path) -> Vault {
        let backend =
            HybridBackend::with_key([3u8; KEY_LEN]).with_store_path(dir.join("store.enc"));
        Vault::with_backend(backend)
    }

    #[test]
    fn context_and_var_roundtrip() {
        let dir = std::env::temp_dir().join(format!("bypass_vault_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let v = vault(&dir);
        v.create_context("dev", "Development").unwrap();
        v.set_var("dev", "TOKEN", "abc").unwrap();

        assert_eq!(v.get_var("dev", "TOKEN").unwrap(), "abc");
        assert_eq!(v.list_keys("dev").unwrap(), vec!["TOKEN"]);
        fs::remove_dir_all(&dir).ok();
    }
}
