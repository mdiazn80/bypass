use std::fs;
use std::io::Write;
use std::process::Command;

use crate::models::Context;

const MANAGED_START: &str = "# ===== BYPASS MANAGED START =====";
const MANAGED_END: &str = "# ===== BYPASS MANAGED END =====";

pub fn hosts_path() -> &'static str {
    "/etc/hosts"
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
        // A pasted copy of a hosts file can carry Bypass's own markers. Written
        // inside the block they would end it early, leaving the rest of the
        // context outside it where disabling the context no longer removes it.
        let lines: Vec<&str> = ctx
            .content
            .trim()
            .lines()
            .filter(|line| !is_marker(line))
            .collect();
        if lines.iter().all(|line| line.trim().is_empty()) {
            continue;
        }
        // Names saved before `clean_context_name` existed may still hold a
        // newline, which would turn the rest of the name into a live entry.
        let label = ctx.name.replace(|c: char| c.is_control(), " ");
        block.push_str(&format!("# >>> Context: \"{}\"\n", label));
        block.push_str(lines.join("\n").trim());
        block.push('\n');
        block.push_str(&format!("# <<< Context: \"{}\"\n", label));
    }

    block.push_str(MANAGED_END);
    block
}

/// Normalises a context name typed by the user. Names are written into comment
/// lines of the hosts file, so control characters become spaces.
pub fn clean_context_name(name: &str) -> Result<String, String> {
    let cleaned = name.replace(|c: char| c.is_control(), " ");
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return Err("Context name cannot be empty".to_string());
    }
    Ok(cleaned.to_string())
}

fn is_marker(line: &str) -> bool {
    let line = line.trim();
    line == MANAGED_START || line == MANAGED_END
}

/// Removes the managed block, keeping every line outside it. A start marker
/// with no end marker is refused: everything after it would otherwise be
/// treated as managed and dropped, user entries included.
fn strip_managed_section(content: &str) -> Result<String, String> {
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

    if inside_managed {
        return Err(format!(
            "The hosts file has a \"{}\" line with no matching \"{}\" line. \
             Fix it by hand before enabling or disabling contexts.",
            MANAGED_START, MANAGED_END
        ));
    }

    // Remove trailing blank lines that accumulated
    let trimmed = result.trim_end_matches('\n');
    let mut final_result = trimmed.to_string();
    final_result.push('\n');
    Ok(final_result)
}

/// The hosts file `current` becomes once the managed block reflects `contexts`.
/// Pure, so the merge can be tested without touching the real hosts file.
fn compose_hosts(current: &str, contexts: &[Context]) -> Result<String, String> {
    let mut new_content = strip_managed_section(current)?;
    let managed = build_managed_block(contexts);
    if !managed.is_empty() {
        new_content.push('\n');
        new_content.push_str(&managed);
        new_content.push('\n');
    }
    Ok(new_content)
}

/// Whether the managed block in `current` is exactly the one `contexts` produce.
/// Lines outside the block are ignored: other tools may add their own entries.
pub fn is_in_sync(current: &str, contexts: &[Context]) -> bool {
    let mut actual: Vec<&str> = Vec::new();
    let mut blocks = 0;
    let mut inside = false;
    for line in current.lines() {
        let trimmed = line.trim();
        if trimmed == MANAGED_START {
            blocks += 1;
            inside = true;
        }
        if inside {
            actual.push(line.trim_end());
        }
        if trimmed == MANAGED_END {
            inside = false;
        }
    }
    if inside || blocks > 1 {
        return false;
    }
    let expected = build_managed_block(contexts);
    actual.iter().copied().eq(expected.lines())
}

pub fn apply_to_hosts(contexts: &[Context]) -> Result<(), String> {
    let current = read_system_hosts()?;
    write_hosts_file(&compose_hosts(&current, contexts)?)
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
        let install = macos_install_script(&temp_str, hosts);
        let applescript = format!(
            "do shell script \"{}\" with administrator privileges",
            install
        );

        // If biometric (Touch ID) is available, authenticate then try sudo -n (non-interactive)
        if crate::biometric::is_available() {
            crate::biometric::authenticate("Bypass needs to modify /etc/hosts")?;

            let output = Command::new("sudo")
                .args(["-n", "/bin/sh", "-c", &install])
                .output()
                .map_err(|e| format!("Failed to run sudo: {}", e))?;

            if !output.status.success() {
                // sudo -n failed (no passwordless sudo), fall back to osascript
                let output = Command::new("osascript")
                    .arg("-e")
                    .arg(&applescript)
                    .output()
                    .map_err(|e| format!("Failed to run osascript: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Failed to write hosts file: {}", stderr));
                }
            }
        } else {
            // No biometric available, use osascript (which may show its own Touch ID prompt)
            let output = Command::new("osascript")
                .arg("-e")
                .arg(&applescript)
                .output()
                .map_err(|e| format!("Failed to run osascript: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to write hosts file: {}", stderr));
            }
        }
    } else {
        fs::remove_file(&temp_path).ok();
        return Err("Writing the hosts file is only supported on macOS".to_string());
    }

    // Clean up temp file
    fs::remove_file(&temp_path).ok();

    Ok(())
}

/// Shell command run as root on macOS: install the new hosts file, then flush
/// the DNS cache so the change is seen at once instead of when entries expire.
/// A failed flush does not fail the write. Contains no double quotes, so it can
/// be embedded as-is in an AppleScript `do shell script` string.
fn macos_install_script(temp_path: &str, hosts: &str) -> String {
    format!(
        "/bin/cp '{}' '{}' && {{ /usr/bin/dscacheutil -flushcache; /usr/bin/killall -HUP mDNSResponder; true; }}",
        temp_path, hosts
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str, content: &str, enabled: bool) -> Context {
        Context {
            id: name.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            enabled,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn compose(current: &str, contexts: &[Context]) -> String {
        compose_hosts(current, contexts).expect("hosts file composes")
    }

    const SYSTEM: &str = "127.0.0.1\tlocalhost\n::1\tlocalhost\n";

    #[test]
    fn block_is_empty_when_no_context_is_enabled() {
        assert_eq!(build_managed_block(&[]), "");
        assert_eq!(build_managed_block(&[ctx("dev", "127.0.0.1 a.local", false)]), "");
    }

    #[test]
    fn block_wraps_each_enabled_context_in_sidebar_order() {
        let block = build_managed_block(&[
            ctx("first", "127.0.0.1 a.local", true),
            ctx("off", "10.0.0.1 b.local", false),
            ctx("second", "  ::1 c.local  \n", true),
        ]);
        assert_eq!(
            block,
            "# ===== BYPASS MANAGED START =====\n\
             # >>> Context: \"first\"\n\
             127.0.0.1 a.local\n\
             # <<< Context: \"first\"\n\
             # >>> Context: \"second\"\n\
             ::1 c.local\n\
             # <<< Context: \"second\"\n\
             # ===== BYPASS MANAGED END ====="
        );
    }

    #[test]
    fn enabled_contexts_with_blank_content_are_skipped() {
        let block = build_managed_block(&[
            ctx("blank", "  \n\n", true),
            ctx("dev", "127.0.0.1 a.local", true),
        ]);
        assert!(!block.contains("blank"));
        assert!(block.contains("# >>> Context: \"dev\""));
    }

    #[test]
    fn appends_block_after_existing_entries() {
        let result = compose(SYSTEM, &[ctx("dev", "127.0.0.1 a.local", true)]);
        assert_eq!(
            result,
            "127.0.0.1\tlocalhost\n::1\tlocalhost\n\n\
             # ===== BYPASS MANAGED START =====\n\
             # >>> Context: \"dev\"\n\
             127.0.0.1 a.local\n\
             # <<< Context: \"dev\"\n\
             # ===== BYPASS MANAGED END =====\n"
        );
    }

    #[test]
    fn replaces_previous_block_instead_of_stacking() {
        let once = compose(SYSTEM, &[ctx("dev", "127.0.0.1 a.local", true)]);
        let twice = compose(&once, &[ctx("dev", "127.0.0.1 b.local", true)]);
        assert_eq!(twice.matches("BYPASS MANAGED START").count(), 1);
        assert!(twice.contains("b.local"));
        assert!(!twice.contains("a.local"));
    }

    #[test]
    fn applying_the_same_contexts_is_idempotent() {
        let contexts = [ctx("dev", "127.0.0.1 a.local", true)];
        let once = compose(SYSTEM, &contexts);
        assert_eq!(compose(&once, &contexts), once);
    }

    #[test]
    fn disabling_everything_restores_the_original_file() {
        let applied = compose(SYSTEM, &[ctx("dev", "127.0.0.1 a.local", true)]);
        assert_eq!(compose(&applied, &[ctx("dev", "127.0.0.1 a.local", false)]), SYSTEM);
    }

    #[test]
    fn keeps_user_lines_on_both_sides_of_the_block() {
        let current = "127.0.0.1 before\n\
                       # ===== BYPASS MANAGED START =====\n\
                       10.0.0.1 stale.local\n\
                       # ===== BYPASS MANAGED END =====\n\
                       127.0.0.1 after\n";
        let result = compose(current, &[]);
        assert_eq!(result, "127.0.0.1 before\n127.0.0.1 after\n");
    }

    #[test]
    fn markers_match_despite_surrounding_whitespace() {
        let current = "127.0.0.1 keep\n  # ===== BYPASS MANAGED START =====  \n10.0.0.1 stale\n# ===== BYPASS MANAGED END =====\t\n";
        assert_eq!(compose(current, &[]), "127.0.0.1 keep\n");
    }

    #[test]
    fn crlf_and_missing_final_newline_are_normalised() {
        let current = "127.0.0.1 a\r\n127.0.0.1 b";
        assert_eq!(compose(current, &[]), "127.0.0.1 a\n127.0.0.1 b\n");
    }

    #[test]
    fn trailing_blank_lines_do_not_accumulate() {
        let current = "127.0.0.1 localhost\n\n\n\n";
        assert_eq!(compose(current, &[]), "127.0.0.1 localhost\n");
    }

    #[test]
    fn start_marker_without_end_is_refused_instead_of_dropping_lines() {
        let current = "127.0.0.1 before\n\
                       # ===== BYPASS MANAGED START =====\n\
                       10.0.0.1 stale.local\n\
                       127.0.0.1 after\n";
        let err = compose_hosts(current, &[ctx("dev", "127.0.0.1 a.local", true)])
            .expect_err("unterminated block must not be rewritten");
        assert!(err.contains("BYPASS MANAGED END"));
    }

    #[test]
    fn markers_inside_context_content_are_not_written() {
        let pasted = "1.1.1.1 x.local\n\
                      # ===== BYPASS MANAGED END =====\n\
                      # ===== BYPASS MANAGED START =====\n\
                      2.2.2.2 y.local";
        let applied = compose(SYSTEM, &[ctx("pasted", pasted, true)]);
        assert_eq!(applied.matches("BYPASS MANAGED START").count(), 1);
        assert_eq!(applied.matches("BYPASS MANAGED END").count(), 1);
        assert!(applied.contains("1.1.1.1 x.local\n2.2.2.2 y.local\n"));
        assert_eq!(compose(&applied, &[]), SYSTEM);
    }

    #[test]
    fn context_made_only_of_markers_is_skipped() {
        let block = build_managed_block(&[
            ctx("markers", "# ===== BYPASS MANAGED END =====", true),
            ctx("dev", "127.0.0.1 a.local", true),
        ]);
        assert!(!block.contains("markers"));
        assert!(block.contains("# >>> Context: \"dev\""));
    }

    #[test]
    fn context_names_lose_control_characters_and_surrounding_space() {
        assert_eq!(clean_context_name("  Dev\n9.9.9.9 evil\t ").unwrap(), "Dev 9.9.9.9 evil");
        assert!(clean_context_name(" \n\t ").is_err());
    }

    #[test]
    fn a_stored_name_with_a_newline_cannot_inject_an_entry() {
        let block = build_managed_block(&[ctx("a\n9.9.9.9 injected", "1.1.1.1 x", true)]);
        assert!(block.lines().all(|line| line.starts_with('#') || line == "1.1.1.1 x"));
    }

    #[test]
    fn in_sync_after_applying_and_when_nothing_is_enabled() {
        let contexts = [ctx("dev", "127.0.0.1 a.local", true)];
        assert!(is_in_sync(&compose(SYSTEM, &contexts), &contexts));
        assert!(is_in_sync(SYSTEM, &[ctx("dev", "127.0.0.1 a.local", false)]));
    }

    #[test]
    fn in_sync_ignores_lines_added_outside_the_block() {
        let contexts = [ctx("dev", "127.0.0.1 a.local", true)];
        let applied = compose(SYSTEM, &contexts) + "10.0.0.5 vpn.internal\n";
        assert!(is_in_sync(&applied, &contexts));
    }

    #[test]
    fn out_of_sync_when_the_block_is_removed_edited_or_unexpected() {
        let contexts = [ctx("dev", "127.0.0.1 a.local", true)];
        let applied = compose(SYSTEM, &contexts);
        assert!(!is_in_sync(SYSTEM, &contexts));
        assert!(!is_in_sync(&applied.replace("a.local", "b.local"), &contexts));
        assert!(!is_in_sync(&applied, &[ctx("dev", "127.0.0.1 a.local", false)]));
        let unterminated = applied.replace("# ===== BYPASS MANAGED END =====", "");
        assert!(!is_in_sync(&unterminated, &contexts));
    }

    #[test]
    fn macos_install_script_flushes_dns_without_failing_the_write() {
        let script = macos_install_script("/tmp/bypass_hosts_tmp", "/etc/hosts");
        assert!(script.starts_with("/bin/cp '/tmp/bypass_hosts_tmp' '/etc/hosts' && "));
        assert!(script.contains("dscacheutil -flushcache"));
        assert!(script.contains("killall -HUP mDNSResponder"));
        assert!(!script.contains('"'));
    }
}
