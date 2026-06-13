use std::collections::BTreeMap;
use std::path::Path;

use crate::backend::{HybridBackend, SecretBackend};
use crate::error::BypassError;
use crate::model::CredentialContext;
use crate::resolver::{self, ResolvedContext};

/// High-level entry point used by both the GUI and the CLI.
///
/// Wraps a [`HybridBackend`] and adds context metadata management and the
/// hierarchical active-context resolution on top of the raw secret CRUD.
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

    /// Builds a vault around an already-constructed backend (tests, fallbacks).
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

    /// Returns all variables of a context as a map for environment injection.
    pub fn vars(&self, context: &str) -> Result<BTreeMap<String, String>, BypassError> {
        self.backend.vars(context)
    }

    // --- Active context -----------------------------------------------------

    pub fn get_active(&self) -> Result<Option<String>, BypassError> {
        self.backend.get_active()
    }

    pub fn set_active(&self, name: Option<&str>) -> Result<(), BypassError> {
        self.backend.set_active(name)
    }

    /// Resolves the effective context for `start`, honoring `.bypass-context`
    /// over the global active context.
    pub fn resolve(&self, start: &Path) -> Result<ResolvedContext, BypassError> {
        let global = self.backend.get_active()?;
        Ok(resolver::resolve_active_context(start, global))
    }

    /// Resolves the effective context and returns its variables. Returns an
    /// empty map when no context is active.
    pub fn resolved_vars(
        &self,
        start: &Path,
    ) -> Result<(ResolvedContext, BTreeMap<String, String>), BypassError> {
        let resolved = self.resolve(start)?;
        let vars = match &resolved.name {
            Some(name) => self.backend.vars(name)?,
            None => BTreeMap::new(),
        };
        Ok((resolved, vars))
    }

    // --- Migration ----------------------------------------------------------

    /// Exports the whole vault to a passphrase-encrypted file.
    pub fn export(&self, path: &Path, passphrase: &str) -> Result<(), BypassError> {
        self.backend.export_to(path, passphrase)
    }

    /// Imports contexts from a passphrase-encrypted file, merging into the
    /// current vault.
    pub fn import(&self, path: &Path, passphrase: &str) -> Result<(), BypassError> {
        self.backend.import_from(path, passphrase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KEY_LEN;
    use std::fs;

    fn vault(dir: &std::path::Path) -> Vault {
        let backend =
            HybridBackend::with_key([3u8; KEY_LEN]).with_store_path(dir.join("store.enc"));
        Vault::with_backend(backend)
    }

    #[test]
    fn resolved_vars_uses_active_context() {
        let dir = std::env::temp_dir().join(format!("bypass_vault_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let v = vault(&dir);
        v.create_context("dev", "Development").unwrap();
        v.set_var("dev", "TOKEN", "abc").unwrap();
        v.set_active(Some("dev")).unwrap();

        let (resolved, vars) = v.resolved_vars(&dir).unwrap();
        assert_eq!(resolved.name.as_deref(), Some("dev"));
        assert_eq!(vars.get("TOKEN").map(String::as_str), Some("abc"));
        fs::remove_dir_all(&dir).ok();
    }
}
