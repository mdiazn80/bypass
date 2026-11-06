use super::backend::{HybridBackend, SecretBackend};
use super::error::BypassError;
use super::files::{self, MAX_FILE_BYTES};
use super::interpolate::{is_valid_name, resolve_all, ResolvedVar};
use super::merge::{merge, MergedVar};
use super::model::{ContextEntry, CredentialContext, FileMeta, FileVar};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// High-level entry point used by the GUI.
///
/// Wraps a [`HybridBackend`] and adds context metadata management on top of the
/// raw secret CRUD.
pub struct Vault {
    backend: HybridBackend,
    /// Where the files of active contexts are written.
    files_root: PathBuf,
}

impl Vault {
    /// Builds a vault backed by the keychain-managed master key.
    pub fn new() -> Result<Self, BypassError> {
        Ok(Self {
            backend: HybridBackend::new()?,
            files_root: files::default_root(),
        })
    }

    /// Builds a vault around an already-constructed backend (tests only).
    #[cfg(test)]
    pub fn with_backend(backend: HybridBackend, files_root: PathBuf) -> Self {
        Self { backend, files_root }
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

    /// Every variable of a context with its `{$VAR}` references resolved against
    /// the other variables of that same context.
    pub fn resolved_vars(&self, context: &str) -> Result<Vec<ResolvedVar>, BypassError> {
        let entry = self.backend.entry(context)?;
        let mut resolved = resolve_all(&self.layer(&entry));
        for var in &mut resolved {
            var.file = entry.files.get(&var.key).map(|f| self.file_meta(&var.key, f));
        }
        Ok(resolved)
    }

    /// The variables of several contexts layered by priority (first wins),
    /// with `{$VAR}` references resolved against the merged set. Contexts that
    /// no longer exist are skipped so a stale active-context name is harmless.
    pub fn merged_vars(&self, contexts: &[String]) -> Result<Vec<MergedVar>, BypassError> {
        let entries = self.backend.entries(contexts)?;
        let layers: Vec<(String, BTreeMap<String, String>)> = entries
            .iter()
            .map(|(name, entry)| (name.clone(), self.layer(entry)))
            .collect();
        let mut merged = merge(&layers);
        for var in &mut merged {
            var.file = entries
                .iter()
                .find(|(name, _)| *name == var.source)
                .and_then(|(_, entry)| entry.files.get(&var.key))
                .map(|f| self.file_meta(&var.key, f));
        }
        Ok(merged)
    }

    /// Stores the file at `path` as a variable of `context`.
    pub fn set_file(&self, context: &str, key: &str, path: &Path) -> Result<(), BypassError> {
        if !is_valid_name(key) {
            return Err(BypassError::Invalid(format!(
                "'{key}' is not a valid variable name"
            )));
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| files::is_valid_filename(n))
            .ok_or_else(|| BypassError::Invalid(format!("unusable file name: {}", path.display())))?;
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(BypassError::Invalid(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(BypassError::Invalid(format!(
                "{filename} is larger than the {} KiB limit",
                MAX_FILE_BYTES / 1024
            )));
        }
        let content = zeroize::Zeroizing::new(fs::read(path)?);
        // The file may have grown between the check above and the read.
        if content.len() as u64 > MAX_FILE_BYTES {
            return Err(BypassError::Invalid(format!(
                "{filename} is larger than the {} KiB limit",
                MAX_FILE_BYTES / 1024
            )));
        }
        self.backend.set_file(context, key, filename, &content)
    }

    /// Writes the winning files of `contexts` to disk and removes all others.
    pub fn sync_files(&self, contexts: &[String]) -> Result<(), BypassError> {
        let entries = self.backend.entries(contexts)?;
        files::sync(&self.files_root, &files::winning_files(&entries)?)?;
        Ok(())
    }

    /// The templates of a context, with each file variable standing in as its
    /// path. A path is literal text, so a `{$` in it must not be expanded.
    fn layer(&self, entry: &ContextEntry) -> BTreeMap<String, String> {
        let mut vars = entry.vars.clone();
        for (key, file) in &entry.files {
            let path = self.file_meta(key, file).path;
            vars.insert(key.clone(), path.replace("{$", "\\{$"));
        }
        vars
    }

    fn file_meta(&self, key: &str, file: &FileVar) -> FileMeta {
        FileMeta {
            filename: file.filename.clone(),
            size: file.size,
            updated_at: file.updated_at.clone(),
            path: files::path_for(&self.files_root, key, &file.filename)
                .to_string_lossy()
                .into_owned(),
        }
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
        Vault::with_backend(backend, dir.join("files"))
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bypass_vault_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_source(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn file_content_is_encrypted_and_materialized_only_on_sync() {
        let dir = temp_dir("file_enc");
        let v = vault(&dir);
        v.create_context("dev", "").unwrap();
        let source = write_source(&dir, "sa.json", b"top-secret-content");
        v.set_file("dev", "GOOGLE_CREDS", &source).unwrap();

        let raw = fs::read(dir.join("store.enc")).unwrap();
        assert!(!raw.windows(10).any(|w| w == b"top-secret"));

        let target = dir.join("files").join("GOOGLE_CREDS").join("sa.json");
        assert!(!target.exists());

        let active = vec!["dev".to_string()];
        let merged = v.merged_vars(&active).unwrap();
        assert_eq!(merged[0].value, target.to_string_lossy());
        assert_eq!(merged[0].file.as_ref().unwrap().filename, "sa.json");
        assert_eq!(merged[0].file.as_ref().unwrap().size, 18);

        v.sync_files(&active).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"top-secret-content");

        // Deactivating the context removes the plaintext copy.
        v.sync_files(&[]).unwrap();
        assert!(!target.exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn other_variables_can_reference_a_file_across_contexts() {
        let dir = temp_dir("file_ref");
        let v = vault(&dir);
        v.create_context("app", "").unwrap();
        v.create_context("infra", "").unwrap();
        v.set_var("app", "OPTS", "--config={$KUBECONFIG}").unwrap();
        let source = write_source(&dir, "kube.yaml", b"x");
        v.set_file("infra", "KUBECONFIG", &source).unwrap();

        let merged = v
            .merged_vars(&["app".to_string(), "infra".to_string()])
            .unwrap();
        let opts = merged.iter().find(|m| m.key == "OPTS").unwrap();
        let path = dir.join("files").join("KUBECONFIG").join("kube.yaml");
        assert_eq!(opts.value, format!("--config={}", path.to_string_lossy()));
        assert!(opts.file.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_reference_like_file_name_is_not_expanded() {
        let dir = temp_dir("file_literal");
        let v = vault(&dir);
        v.create_context("dev", "").unwrap();
        let source = write_source(&dir, "a{$B}.txt", b"x");
        v.set_file("dev", "F", &source).unwrap();

        let resolved = v.resolved_vars("dev").unwrap();
        assert!(resolved[0].value.ends_with("a{$B}.txt"));
        assert!(resolved[0].issue.is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_key_is_either_a_file_or_a_plain_variable() {
        let dir = temp_dir("file_swap");
        let v = vault(&dir);
        v.create_context("dev", "").unwrap();
        let source = write_source(&dir, "f", b"x");

        v.set_var("dev", "K", "plain").unwrap();
        v.set_file("dev", "K", &source).unwrap();
        assert_eq!(v.list_keys("dev").unwrap(), vec!["K"]);
        assert!(v.get_var("dev", "K").is_err());

        v.set_var("dev", "K", "plain").unwrap();
        assert_eq!(v.get_var("dev", "K").unwrap(), "plain");
        assert_eq!(v.list_keys("dev").unwrap(), vec!["K"]);
        assert!(v.resolved_vars("dev").unwrap()[0].file.is_none());

        v.set_file("dev", "K", &source).unwrap();
        v.delete_var("dev", "K").unwrap();
        assert!(v.list_keys("dev").unwrap().is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn set_file_rejects_bad_input() {
        let dir = temp_dir("file_bad");
        let v = vault(&dir);
        v.create_context("dev", "").unwrap();
        let ok = write_source(&dir, "ok", b"x");
        let big = write_source(&dir, "big", &vec![0u8; MAX_FILE_BYTES as usize + 1]);

        // The key becomes a directory name, so it must not be able to escape.
        assert!(v.set_file("dev", "../../evil", &ok).is_err());
        assert!(v.set_file("dev", "K", &big).is_err());
        assert!(v.set_file("dev", "K", &dir).is_err());
        assert!(v.set_file("dev", "K", &dir.join("missing")).is_err());
        assert!(v.set_file("missing", "K", &ok).is_err());
        assert!(v.list_keys("dev").unwrap().is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn stores_without_files_still_load() {
        let entry: ContextEntry = serde_json::from_str(r#"{"vars":{"A":"1"}}"#).unwrap();
        assert!(entry.files.is_empty());
        assert_eq!(entry.vars["A"], "1");
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

    #[test]
    fn resolved_vars_expands_references_but_get_var_stays_raw() {
        let dir = std::env::temp_dir().join(format!("bypass_vault_ref_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let v = vault(&dir);
        v.create_context("dev", "Development").unwrap();
        v.set_var("dev", "APP_PATH", "/opt/app").unwrap();
        v.set_var("dev", "APP_PATH_CONFIG", "{$APP_PATH}/config")
            .unwrap();

        // The editor must keep seeing the template it can edit.
        assert_eq!(
            v.get_var("dev", "APP_PATH_CONFIG").unwrap(),
            "{$APP_PATH}/config"
        );

        let resolved = v.resolved_vars("dev").unwrap();
        let cfg = resolved
            .iter()
            .find(|r| r.key == "APP_PATH_CONFIG")
            .unwrap();
        assert_eq!(cfg.value, "/opt/app/config");
        assert!(cfg.issue.is_none());

        // Changing the referenced variable updates the dependant one.
        v.set_var("dev", "APP_PATH", "/srv/app").unwrap();
        let resolved = v.resolved_vars("dev").unwrap();
        let cfg = resolved
            .iter()
            .find(|r| r.key == "APP_PATH_CONFIG")
            .unwrap();
        assert_eq!(cfg.value, "/srv/app/config");

        fs::remove_dir_all(&dir).ok();
    }
}
