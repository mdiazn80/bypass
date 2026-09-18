use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub id: String,
    pub name: String,
    pub content: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub start_minimized: bool,
    /// Whether the local shell agent (socket listener) should run.
    #[serde(default)]
    pub shell_integration_enabled: bool,
    /// Whether the prompt hook has been written into the user's shell rc.
    #[serde(default)]
    pub shell_integration_installed: bool,
    /// Legacy single active context. Read for migration only; `load_config`
    /// folds it into `active_contexts` and it is never written back.
    #[serde(default, skip_serializing)]
    pub active_context: Option<String>,
    /// Credential contexts whose variables are served to shells. Several can
    /// be active at once; see `credential_order` for which one wins.
    #[serde(default)]
    pub active_contexts: Vec<String>,
    /// Display and priority order of the credential contexts. When two active
    /// contexts define the same variable, the one listed first wins. Names not
    /// listed here sort after the listed ones, alphabetically.
    #[serde(default)]
    pub credential_order: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            start_minimized: false,
            shell_integration_enabled: false,
            shell_integration_installed: false,
            active_context: None,
            active_contexts: Vec::new(),
            credential_order: Vec::new(),
        }
    }
}

impl AppConfig {
    /// Position of `name` in the priority order; unlisted names sort last.
    pub fn credential_rank(&self, name: &str) -> usize {
        self.credential_order
            .iter()
            .position(|n| n == name)
            .unwrap_or(usize::MAX)
    }

    /// Active credential contexts, highest priority first.
    pub fn active_contexts_by_priority(&self) -> Vec<String> {
        let mut active = self.active_contexts.clone();
        active.sort_by(|a, b| {
            self.credential_rank(a)
                .cmp(&self.credential_rank(b))
                .then_with(|| a.cmp(b))
        });
        active
    }

    /// Applies a credential context rename to both lists.
    pub fn rename_credential(&mut self, old: &str, new: &str) {
        for n in self
            .active_contexts
            .iter_mut()
            .chain(self.credential_order.iter_mut())
        {
            if n == old {
                *n = new.to_string();
            }
        }
    }

    /// Forgets a deleted credential context.
    pub fn forget_credential(&mut self, name: &str) {
        self.active_contexts.retain(|n| n != name);
        self.credential_order.retain(|n| n != name);
    }
}
