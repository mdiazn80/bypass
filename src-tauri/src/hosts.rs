use std::fs;
use std::io::Write;
use std::process::Command;

use crate::models::Context;

const MANAGED_START: &str = "# ===== BYPASS MANAGED START =====";
const MANAGED_END: &str = "# ===== BYPASS MANAGED END =====";

fn hosts_path() -> &'static str {
    if cfg!(target_os = "windows") {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    }
}

pub fn read_system_hosts() -> Result<String, String> {
    fs::read_to_string(hosts_path()).map_err(|e| format!("Failed to read hosts file: {}", e))
}

pub fn build_managed_block(contexts: &[Context]) -> String {
    let enabled: Vec<&Context> = contexts.iter().filter(|c| c.enabled).collect();
    if enabled.is_empty() {
        return String::new();
    }

    let mut block = String::new();
    block.push_str(MANAGED_START);
    block.push('\n');

    for ctx in &enabled {
        let content = ctx.content.trim();
        if content.is_empty() {
            continue;
        }
        block.push_str(&format!("# >>> Context: \"{}\"\n", ctx.name));
        block.push_str(content);
        block.push('\n');
        block.push_str(&format!("# <<< Context: \"{}\"\n", ctx.name));
    }

    block.push_str(MANAGED_END);
    block
}

fn strip_managed_section(content: &str) -> String {
    let mut result = String::new();
    let mut inside_managed = false;

    for line in content.lines() {
        if line.trim() == MANAGED_START {
            inside_managed = true;
            continue;
        }
        if line.trim() == MANAGED_END {
            inside_managed = false;
            continue;
        }
        if !inside_managed {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Remove trailing blank lines that accumulated
    let trimmed = result.trim_end_matches('\n');
    let mut final_result = trimmed.to_string();
    final_result.push('\n');
    final_result
}

pub fn apply_to_hosts(contexts: &[Context]) -> Result<(), String> {
    let current = read_system_hosts()?;
    let base = strip_managed_section(&current);
    let managed = build_managed_block(contexts);

    let mut new_content = base.clone();
    if !managed.is_empty() {
        new_content.push('\n');
        new_content.push_str(&managed);
        new_content.push('\n');
    }

    write_hosts_file(&new_content)
}

fn write_hosts_file(content: &str) -> Result<(), String> {
    // Write to a temp file first, then copy with elevated privileges
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("bypass_hosts_tmp");
    let temp_str = temp_path.to_string_lossy().to_string();

    {
        let mut f =
            fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
        f.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;
    }

    let hosts = hosts_path();

    if cfg!(target_os = "macos") {
        // If biometric (Touch ID) is available, authenticate then try sudo -n (non-interactive)
        if crate::biometric::is_available() {
            crate::biometric::authenticate("Bypass needs to modify /etc/hosts")?;

            let output = Command::new("sudo")
                .args(["-n", "cp", &temp_str, hosts])
                .output()
                .map_err(|e| format!("Failed to run sudo: {}", e))?;

            if !output.status.success() {
                // sudo -n failed (no passwordless sudo), fall back to osascript
                let script = format!(
                    "do shell script \"cp '{}' '{}'\" with administrator privileges",
                    temp_str, hosts
                );
                let output = Command::new("osascript")
                    .arg("-e")
                    .arg(&script)
                    .output()
                    .map_err(|e| format!("Failed to run osascript: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Failed to write hosts file: {}", stderr));
                }
            }
        } else {
            // No biometric available, use osascript (which may show its own Touch ID prompt)
            let script = format!(
                "do shell script \"cp '{}' '{}'\" with administrator privileges",
                temp_str, hosts
            );
            let output = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("Failed to run osascript: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to write hosts file: {}", stderr));
            }
        }
    } else if cfg!(target_os = "linux") {
        let output = Command::new("pkexec")
            .args(["cp", &temp_str, hosts])
            .output()
            .map_err(|e| format!("Failed to run pkexec: {}", e))?;

        if !output.status.success() {
            // Fallback to sudo
            let output = Command::new("sudo")
                .args(["cp", &temp_str, hosts])
                .output()
                .map_err(|e| format!("Failed to run sudo: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to write hosts file: {}", stderr));
            }
        }
    } else {
        // Windows: direct write (app should run as admin)
        fs::copy(&temp_path, hosts)
            .map_err(|e| format!("Failed to write hosts file: {}", e))?;
    }

    // Clean up temp file
    fs::remove_file(&temp_path).ok();

    Ok(())
}
