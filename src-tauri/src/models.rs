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
    /// Name of the credential context whose variables are served to shells.
    #[serde(default)]
    pub active_context: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            start_minimized: false,
            shell_integration_enabled: false,
            shell_integration_installed: false,
            active_context: None,
        }
    }
}
