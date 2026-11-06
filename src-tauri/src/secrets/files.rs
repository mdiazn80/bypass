//! File variables: a file whose content lives encrypted in the vault and whose
//! path is what the variable resolves to.
//!
//! Tools such as `KUBECONFIG` or `GOOGLE_APPLICATION_CREDENTIALS` need a real
//! path, so the winning file of each key is written to disk for as long as its
//! context is active and the shell agent runs. This is the one place where
//! vault content leaves `store.enc` unencrypted, and it is kept to a minimum:
//! private permissions, excluded from Spotlight and Time Machine, removed on
//! deactivation and on exit, and swept at startup in case of a crash.
//!
//! The path depends only on the key and the file name, never on the context.
//! Only one context can own a key at a time, so paths cannot collide, and
//! switching contexts updates the content in place for running processes.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use base64::Engine;
use zeroize::Zeroizing;

use super::error::BypassError;
use super::model::ContextEntry;

/// Largest file the vault accepts. The whole store is decrypted on every
/// operation, so this is about keeping the vault fast: certificates, service
/// account keys and kubeconfigs are far smaller.
pub(crate) const MAX_FILE_BYTES: u64 = 1 << 20;

/// A file to be written to disk.
pub(crate) struct Materialized {
    pub key: String,
    pub filename: String,
    pub bytes: Zeroizing<Vec<u8>>,
}

/// Where active files are written. The `.noindex` suffix keeps Spotlight out.
pub(crate) fn default_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".bypass")
        .join("files.noindex")
}

pub(crate) fn path_for(root: &Path, key: &str, filename: &str) -> PathBuf {
    root.join(key).join(filename)
}

/// Rejects anything that is not a single, plain path component. Keys are checked
/// separately, as environment-variable names.
pub(crate) fn is_valid_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.chars().any(|c| c == '/' || c == '\\' || c.is_control())
}

/// Picks the file that owns each key across `entries` (highest priority first).
/// A plain variable in a higher-priority context shadows a lower file, matching
/// what `merge` serves.
pub(crate) fn winning_files(
    entries: &[(String, ContextEntry)],
) -> Result<Vec<Materialized>, BypassError> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for (_, entry) in entries {
        for (key, file) in &entry.files {
            if seen.contains(key.as_str()) {
                continue;
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&file.content)
                .map_err(|e| BypassError::Invalid(format!("file variable {key}: {e}")))?;
            out.push(Materialized {
                key: key.clone(),
                filename: file.filename.clone(),
                bytes: Zeroizing::new(bytes),
            });
        }
        seen.extend(entry.vars.keys().map(String::as_str));
        seen.extend(entry.files.keys().map(String::as_str));
    }
    Ok(out)
}

/// Makes the contents of `root` exactly `files`.
pub(crate) fn sync(root: &Path, files: &[Materialized]) -> io::Result<()> {
    if files.is_empty() && !root.exists() {
        return Ok(());
    }
    let created = !root.exists();
    create_private_dir(root)?;
    if created {
        exclude_from_backups(root);
    }
    for file in files {
        write_file(root, file)?;
    }
    prune(root, files)
}

/// Removes every materialized file but keeps `root`, so its backup exclusion
/// survives.
pub(crate) fn clear(root: &Path) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    prune(root, &[])
}

/// Same as [`clear`] on the default location. Needs no vault, so startup and
/// exit never trigger a keychain prompt.
pub fn clear_materialized() -> io::Result<()> {
    clear(&default_root())
}

fn write_file(root: &Path, file: &Materialized) -> io::Result<()> {
    let dir = root.join(&file.key);
    create_private_dir(&dir)?;
    let target = dir.join(&file.filename);
    // Unchanged content keeps its mtime, so tools watching the file stay quiet.
    let unchanged = fs::read(&target)
        .map(|current| Zeroizing::new(current).as_slice() == file.bytes.as_slice())
        .unwrap_or(false);
    if unchanged {
        return Ok(());
    }

    // Written aside and renamed, so a running process never reads half a file.
    let temp = dir.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let written = write_private(&temp, &file.bytes).and_then(|()| fs::rename(&temp, &target));
    if written.is_err() {
        let _ = fs::remove_file(&temp);
    }
    written
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Deletes whatever under `root` is not in `files`.
fn prune(root: &Path, files: &[Materialized]) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let wanted = files.iter().find(|f| name == f.key.as_str());
        match wanted {
            Some(file) if entry.file_type()?.is_dir() => {
                for inner in fs::read_dir(entry.path())? {
                    let inner = inner?;
                    if inner.file_name() != file.filename.as_str() {
                        remove_entry(&inner)?;
                    }
                }
            }
            _ => remove_entry(&entry)?,
        }
    }
    Ok(())
}

fn remove_entry(entry: &fs::DirEntry) -> io::Result<()> {
    if entry.file_type()?.is_dir() {
        fs::remove_dir_all(entry.path())
    } else {
        fs::remove_file(entry.path())
    }
}

/// Best effort: a failure here must not stop the variable from working, since
/// the user may simply not use Time Machine.
#[cfg(all(target_os = "macos", not(test)))]
fn exclude_from_backups(root: &Path) {
    let status = std::process::Command::new("/usr/bin/tmutil")
        .arg("addexclusion")
        .arg(root)
        .status();
    if !matches!(status, Ok(s) if s.success()) {
        eprintln!("could not exclude {} from Time Machine", root.display());
    }
}

#[cfg(not(all(target_os = "macos", not(test))))]
fn exclude_from_backups(_root: &Path) {}

#[cfg(test)]
mod tests {
    use super::super::model::FileVar;
    use super::*;
    use std::collections::BTreeMap;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bypass_files_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn materialized(key: &str, filename: &str, content: &[u8]) -> Materialized {
        Materialized {
            key: key.to_string(),
            filename: filename.to_string(),
            bytes: Zeroizing::new(content.to_vec()),
        }
    }

    fn file_var(filename: &str, content: &[u8]) -> FileVar {
        FileVar {
            filename: filename.to_string(),
            content: base64::engine::general_purpose::STANDARD.encode(content),
            size: content.len() as u64,
            updated_at: String::new(),
        }
    }

    fn entry(vars: &[&str], files: &[(&str, FileVar)]) -> ContextEntry {
        ContextEntry {
            vars: vars.iter().map(|k| (k.to_string(), "v".to_string())).collect(),
            files: files
                .iter()
                .map(|(k, f)| (k.to_string(), f.clone()))
                .collect::<BTreeMap<_, _>>(),
            ..ContextEntry::default()
        }
    }

    #[test]
    fn sync_writes_binary_content_with_private_permissions() {
        let root = temp_root("write");
        let content = [0u8, 159, 146, 150];
        sync(&root, &[materialized("CERT", "client.p12", &content)]).unwrap();

        let path = path_for(&root, "CERT", "client.p12");
        assert_eq!(fs::read(&path).unwrap(), content);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode(&path), 0o600);
            assert_eq!(mode(path.parent().unwrap()), 0o700);
            assert_eq!(mode(&root), 0o700);
        }
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sync_updates_in_place_and_prunes_the_rest() {
        let root = temp_root("prune");
        sync(
            &root,
            &[
                materialized("A", "a.json", b"one"),
                materialized("B", "old.pem", b"gone"),
            ],
        )
        .unwrap();
        sync(&root, &[materialized("A", "a.json", b"two")]).unwrap();

        assert_eq!(fs::read(path_for(&root, "A", "a.json")).unwrap(), b"two");
        assert!(!root.join("B").exists());

        // Renaming the file within a key removes the previous name.
        sync(&root, &[materialized("A", "b.json", b"two")]).unwrap();
        assert!(!path_for(&root, "A", "a.json").exists());
        assert!(path_for(&root, "A", "b.json").exists());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn clear_removes_files_but_keeps_the_root() {
        let root = temp_root("clear");
        sync(&root, &[materialized("A", "a", b"x")]).unwrap();
        clear(&root).unwrap();
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sync_without_files_does_not_create_the_root() {
        let root = temp_root("empty");
        sync(&root, &[]).unwrap();
        clear(&root).unwrap();
        assert!(!root.exists());
    }

    #[test]
    fn highest_priority_file_wins() {
        let entries = vec![
            ("prod".to_string(), entry(&[], &[("K", file_var("prod.yaml", b"p"))])),
            ("dev".to_string(), entry(&[], &[("K", file_var("dev.yaml", b"d"))])),
        ];
        let files = winning_files(&entries).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "prod.yaml");
        assert_eq!(files[0].bytes.as_slice(), b"p");
    }

    #[test]
    fn higher_priority_plain_variable_shadows_a_file() {
        let entries = vec![
            ("override".to_string(), entry(&["K"], &[])),
            ("base".to_string(), entry(&[], &[("K", file_var("k", b"x"))])),
        ];
        assert!(winning_files(&entries).unwrap().is_empty());
    }

    #[test]
    fn filename_validation() {
        assert!(is_valid_filename("service-account.json"));
        assert!(is_valid_filename("my cert.pem"));
        assert!(!is_valid_filename(""));
        assert!(!is_valid_filename(".."));
        assert!(!is_valid_filename("../evil"));
        assert!(!is_valid_filename("a/b"));
        assert!(!is_valid_filename("a\nb"));
    }
}
