use std::fs;
use std::path::{Path, PathBuf};

/// Where the resolved active context came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionSource {
    /// A `.bypass-context` file found at the given path.
    ProjectFile(PathBuf),
    /// The globally configured active context.
    Global,
    /// No active context could be determined.
    None,
}

/// Outcome of resolving the active context for a working directory.
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub name: Option<String>,
    pub source: ResolutionSource,
}

/// Walks up from `start` looking for a `.bypass-context` file, the same way
/// git discovers a `.git` directory. Returns the file path and the context
/// name it contains (first non-empty trimmed line).
pub fn find_project_context(start: &Path) -> Option<(PathBuf, String)> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join(".bypass-context");
        if candidate.is_file() {
            if let Ok(content) = fs::read_to_string(&candidate) {
                if let Some(name) = content.lines().map(str::trim).find(|l| !l.is_empty()) {
                    return Some((candidate, name.to_string()));
                }
            }
        }
        current = dir.parent();
    }
    None
}

/// Resolves the active context with project-file priority over the global
/// active context.
pub fn resolve_active_context(start: &Path, global_active: Option<String>) -> ResolvedContext {
    if let Some((path, name)) = find_project_context(start) {
        return ResolvedContext {
            name: Some(name),
            source: ResolutionSource::ProjectFile(path),
        };
    }
    match global_active {
        Some(name) => ResolvedContext {
            name: Some(name),
            source: ResolutionSource::Global,
        },
        None => ResolvedContext {
            name: None,
            source: ResolutionSource::None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_file_takes_priority() {
        let base = std::env::temp_dir().join(format!("bypass_resolver_{}", std::process::id()));
        let nested = base.join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(base.join("a/.bypass-context"), "project-ctx\n").unwrap();

        let resolved = resolve_active_context(&nested, Some("global-ctx".to_string()));
        assert_eq!(resolved.name.as_deref(), Some("project-ctx"));
        assert!(matches!(resolved.source, ResolutionSource::ProjectFile(_)));
        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn falls_back_to_global() {
        let dir = std::env::temp_dir().join(format!("bypass_resolver_g_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let resolved = resolve_active_context(&dir, Some("global-ctx".to_string()));
        assert_eq!(resolved.name.as_deref(), Some("global-ctx"));
        assert_eq!(resolved.source, ResolutionSource::Global);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn none_when_nothing_set() {
        let dir = std::env::temp_dir().join(format!("bypass_resolver_n_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let resolved = resolve_active_context(&dir, None);
        assert_eq!(resolved.name, None);
        assert_eq!(resolved.source, ResolutionSource::None);
        fs::remove_dir_all(&dir).ok();
    }
}
