//! Installs/removes the prompt hook that calls `bypass-shell` into the user's
//! shell startup files. Idempotent: a marked block is rewritten in place.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

const BEGIN: &str = "# >>> bypass shell integration >>>";
const END: &str = "# <<< bypass shell integration <<<";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DetectedShell {
    Zsh,
    Bash,
    Fish,
    // Only constructed on Windows; detection elsewhere never yields it.
    #[cfg_attr(not(windows), allow(dead_code))]
    Powershell,
}

impl DetectedShell {
    /// The `--shell` value passed to `bypass-shell emit`.
    fn emit_name(self) -> &'static str {
        match self {
            DetectedShell::Zsh => "zsh",
            DetectedShell::Bash => "bash",
            DetectedShell::Fish => "fish",
            DetectedShell::Powershell => "powershell",
        }
    }

    fn label(self) -> &'static str {
        self.emit_name()
    }
}

/// Snapshot of the integration state, surfaced to the Settings UI.
#[derive(Serialize)]
pub struct ShellStatus {
    pub enabled: bool,
    pub installed: bool,
    pub socket_active: bool,
    pub active_context: Option<String>,
    pub detected_shell: Option<String>,
    pub rc_path: Option<String>,
}

/// Detects the user's interactive shell. On Windows we always target PowerShell.
pub fn detect() -> Option<DetectedShell> {
    #[cfg(windows)]
    {
        return Some(DetectedShell::Powershell);
    }
    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.ends_with("zsh") {
            Some(DetectedShell::Zsh)
        } else if shell.ends_with("bash") {
            Some(DetectedShell::Bash)
        } else if shell.ends_with("fish") {
            Some(DetectedShell::Fish)
        } else {
            None
        }
    }
}

pub fn detected_shell_label() -> Option<String> {
    detect().map(|s| s.label().to_string())
}

pub fn rc_path_string() -> Option<String> {
    detect()
        .and_then(rc_path)
        .map(|p| p.to_string_lossy().into_owned())
}

/// The startup file the hook is written into for `shell`.
fn rc_path(shell: DetectedShell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match shell {
        DetectedShell::Zsh => Some(home.join(".zshrc")),
        DetectedShell::Bash => Some(home.join(".bashrc")),
        DetectedShell::Fish => Some(home.join(".config").join("fish").join("conf.d").join("bypass.fish")),
        DetectedShell::Powershell => powershell_profile_path(),
    }
}

#[cfg(windows)]
fn powershell_profile_path() -> Option<PathBuf> {
    // Documents\PowerShell\Microsoft.PowerShell_profile.ps1
    dirs::document_dir().map(|d| {
        d.join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1")
    })
}

#[cfg(not(windows))]
fn powershell_profile_path() -> Option<PathBuf> {
    // PowerShell Core on Unix: ~/.config/powershell/Microsoft.PowerShell_profile.ps1
    dirs::home_dir().map(|h| {
        h.join(".config")
            .join("powershell")
            .join("Microsoft.PowerShell_profile.ps1")
    })
}

/// Where the client binary is copied so the snippet can reference a stable path.
fn installed_client_path() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "bypass-shell.exe"
    } else {
        "bypass-shell"
    };
    Some(dirs::home_dir()?.join(".bypass").join("bin").join(name))
}

/// Locates the bundled/built client binary to copy from.
fn source_client_path(app: &AppHandle) -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "bypass-shell.exe"
    } else {
        "bypass-shell"
    };
    // 1. Next to the running executable (dev: target/debug; bundled: alongside).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    // 2. Bundled resource directory.
    if let Ok(res) = app.path().resource_dir() {
        let candidate = res.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Copies the client binary into `~/.bypass/bin` and returns its path.
fn ensure_client_installed(app: &AppHandle) -> Result<PathBuf, String> {
    let src = source_client_path(app)
        .ok_or_else(|| "bypass-shell client binary not found".to_string())?;
    let dest = installed_client_path().ok_or_else(|| "no home dir".to_string())?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&src, &dest).map_err(|e| format!("failed to copy client: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
    }
    Ok(dest)
}

/// The hook code for `shell`, referencing the absolute `client` path.
fn snippet(shell: DetectedShell, client: &Path) -> String {
    let c = client.to_string_lossy();
    match shell {
        // precmd refreshes at the next prompt; preexec refreshes right before a
        // command runs, so `echo $VAR` typed immediately after a value change in
        // the app already sees the new value (no one-prompt lag).
        DetectedShell::Zsh => format!(
            "{BEGIN}\n\
             _bypass_sync() {{ eval \"$(\"{c}\" emit --shell zsh)\"; }}\n\
             typeset -ag precmd_functions preexec_functions\n\
             if [[ -z \"${{precmd_functions[(r)_bypass_sync]}}\" ]]; then precmd_functions+=(_bypass_sync); fi\n\
             if [[ -z \"${{preexec_functions[(r)_bypass_sync]}}\" ]]; then preexec_functions+=(_bypass_sync); fi\n\
             {END}\n"
        ),
        DetectedShell::Bash => format!(
            "{BEGIN}\n\
             _bypass_sync() {{ eval \"$(\"{c}\" emit --shell bash)\"; }}\n\
             case \"$PROMPT_COMMAND\" in *_bypass_sync*) ;; *) PROMPT_COMMAND=\"_bypass_sync;${{PROMPT_COMMAND}}\" ;; esac\n\
             case \"$(trap -p DEBUG)\" in *_bypass_sync*) ;; \"\") trap '_bypass_sync' DEBUG ;; *) ;; esac\n\
             {END}\n"
        ),
        DetectedShell::Fish => format!(
            "{BEGIN}\n\
             function _bypass_apply\n\
             \x20   \"{c}\" emit --shell fish | source\n\
             end\n\
             function _bypass_on_prompt --on-event fish_prompt\n\
             \x20   _bypass_apply\n\
             end\n\
             function _bypass_on_preexec --on-event fish_preexec\n\
             \x20   _bypass_apply\n\
             end\n\
             {END}\n"
        ),
        DetectedShell::Powershell => format!(
            "{BEGIN}\n\
             function global:prompt {{\n\
             \x20   Invoke-Expression (& \"{c}\" emit --shell powershell | Out-String)\n\
             \x20   \"PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) \"\n\
             }}\n\
             {END}\n"
        ),
    }
}

/// Installs the hook for the detected shell.
pub fn install(app: &AppHandle) -> Result<(), String> {
    let shell = detect().ok_or_else(|| "Could not detect a supported shell".to_string())?;
    let rc = rc_path(shell).ok_or_else(|| "Could not resolve shell rc path".to_string())?;
    let client = ensure_client_installed(app)?;
    let block = snippet(shell, &client);

    if shell == DetectedShell::Fish {
        // Dedicated conf.d file: write it whole.
        if let Some(parent) = rc.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&rc, block).map_err(|e| e.to_string())?;
    } else {
        write_marked_block(&rc, &block)?;
    }
    Ok(())
}

/// Removes the hook for the detected shell.
pub fn uninstall() -> Result<(), String> {
    let shell = detect().ok_or_else(|| "Could not detect a supported shell".to_string())?;
    let rc = match rc_path(shell) {
        Some(p) => p,
        None => return Ok(()),
    };
    if shell == DetectedShell::Fish {
        let _ = fs::remove_file(&rc);
    } else {
        strip_marked_block(&rc)?;
    }
    Ok(())
}

/// Replaces any existing marked block in `path` with `block`, creating the file
/// if needed. Content outside the markers is preserved.
fn write_marked_block(path: &Path, block: &str) -> Result<(), String> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let base = remove_block(&existing);
    let mut out = base.trim_end().to_string();
    if !out.is_empty() {
        // Separate prior content from our block with one blank line.
        out.push_str("\n\n");
    }
    out.push_str(block);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, out).map_err(|e| e.to_string())
}

fn strip_marked_block(path: &Path) -> Result<(), String> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    let cleaned = remove_block(&existing);
    fs::write(path, cleaned.trim_end().to_string() + "\n").map_err(|e| e.to_string())
}

/// Drops the `BEGIN..END` block (inclusive) from `content` if present.
fn remove_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let begin = lines.iter().position(|l| l.trim() == BEGIN);
    let end = lines.iter().position(|l| l.trim() == END);
    if let (Some(b), Some(e)) = (begin, end) {
        if b <= e {
            let mut kept: Vec<&str> = Vec::new();
            kept.extend_from_slice(&lines[..b]);
            kept.extend_from_slice(&lines[e + 1..]);
            return kept.join("\n");
        }
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_roundtrip_is_idempotent() {
        let base = "export FOO=1\nalias ll='ls -la'\n";
        let block = format!("{BEGIN}\nhook line\n{END}\n");

        let with_block = {
            let cleaned = remove_block(base);
            format!("{}\n\n{}", cleaned.trim_end(), block)
        };
        // Installing again must not duplicate the block.
        let reinstalled = {
            let cleaned = remove_block(&with_block);
            format!("{}\n\n{}", cleaned.trim_end(), block)
        };
        assert_eq!(with_block, reinstalled);
        assert!(reinstalled.contains("export FOO=1"));
        assert!(reinstalled.contains("alias ll='ls -la'"));
        assert_eq!(reinstalled.matches(BEGIN).count(), 1);
    }

    #[test]
    fn remove_block_preserves_surrounding_content() {
        let content = format!("a\n{BEGIN}\nx\ny\n{END}\nb\n");
        let cleaned = remove_block(&content);
        assert!(cleaned.contains('a'));
        assert!(cleaned.contains('b'));
        assert!(!cleaned.contains('x'));
        assert!(!cleaned.contains(BEGIN));
    }
}
