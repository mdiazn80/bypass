//! Bypass shell-integration client.
//!
//! Invoked by a shell prompt hook on every prompt. It connects to the local
//! Bypass agent, asks for the active context's variables (only when they
//! changed since last time), and prints shell code to stdout that the hook
//! `eval`s. If the agent is not running it prints nothing and exits 0 so the
//! prompt is never blocked.

use interprocess::local_socket::traits::Stream as _;
use std::io::{Read, Write};

mod socket;

/// Target shell dialect for the emitted code.
#[derive(Clone, Copy)]
enum Shell {
    /// POSIX-ish: zsh and bash.
    Sh,
    Fish,
    Powershell,
}

fn main() {
    let shell = match parse_args() {
        Some(s) => s,
        None => {
            eprintln!("usage: bypass-shell emit --shell <zsh|bash|fish|powershell>");
            std::process::exit(2);
        }
    };

    // Whatever happens, never fail loudly: a broken prompt is worse than a
    // missing variable. On any error we simply print nothing.
    match run(shell) {
        Ok(out) => print!("{out}"),
        Err(_) => { /* silent degrade */ }
    }
}

/// Parses `emit --shell <name>`. Returns None on any malformed input.
fn parse_args() -> Option<Shell> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("emit") {
        return None;
    }
    let mut shell = None;
    while let Some(arg) = args.next() {
        if arg == "--shell" {
            shell = args.next();
        }
    }
    match shell.as_deref() {
        Some("zsh") | Some("bash") | Some("sh") => Some(Shell::Sh),
        Some("fish") => Some(Shell::Fish),
        Some("powershell") | Some("pwsh") => Some(Shell::Powershell),
        _ => None,
    }
}

fn run(shell: Shell) -> Result<String, Box<dyn std::error::Error>> {
    let gen = std::env::var("_BYPASS_GEN")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let old_keys: Vec<String> = std::env::var("_BYPASS_KEYS")
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect();

    // Connect to the agent. Any failure (not running, stale socket) is silent.
    let name = socket::name()?;
    let mut conn = interprocess::local_socket::Stream::connect(name)?;
    conn.write_all(format!("SYNC {gen}\n").as_bytes())?;
    conn.flush()?;

    let mut resp = String::new();
    conn.read_to_string(&mut resp)?;

    let mut lines = resp.lines();
    match lines.next() {
        Some("UNCHANGED") => return Ok(String::new()),
        Some(first) if first.starts_with("GEN ") => {
            let new_gen: u64 = first[4..].trim().parse().unwrap_or(gen);
            let mut vars: Vec<(String, String)> = Vec::new();
            for line in lines {
                if line == "END" {
                    break;
                }
                // VAR <KEY> <base64(value)>
                if let Some(rest) = line.strip_prefix("VAR ") {
                    if let Some((key, b64)) = rest.split_once(' ') {
                        if !is_valid_name(key) {
                            continue;
                        }
                        if let Ok(bytes) = base64_decode(b64) {
                            if let Ok(value) = String::from_utf8(bytes) {
                                vars.push((key.to_string(), value));
                            }
                        }
                    }
                }
            }
            Ok(emit(shell, new_gen, &vars, &old_keys))
        }
        _ => Ok(String::new()),
    }
}

/// Builds the shell code: unset removed keys, export current ones, and update
/// the `_BYPASS_GEN` / `_BYPASS_KEYS` bookkeeping (which must be exported so the
/// next invocation of this binary can read them).
fn emit(shell: Shell, gen: u64, vars: &[(String, String)], old_keys: &[String]) -> String {
    let new_keys: Vec<&str> = vars.iter().map(|(k, _)| k.as_str()).collect();
    let to_unset: Vec<&String> = old_keys
        .iter()
        .filter(|k| !new_keys.contains(&k.as_str()))
        .collect();
    let keys_joined = new_keys.join(" ");

    let mut out = String::new();
    match shell {
        Shell::Sh => {
            for k in &to_unset {
                out.push_str(&format!("unset {k}\n"));
            }
            for (k, v) in vars {
                out.push_str(&format!("export {k}={}\n", quote_sh(v)));
            }
            out.push_str(&format!("export _BYPASS_GEN={gen}\n"));
            out.push_str(&format!("export _BYPASS_KEYS={}\n", quote_sh(&keys_joined)));
        }
        Shell::Fish => {
            for k in &to_unset {
                out.push_str(&format!("set -e {k}\n"));
            }
            for (k, v) in vars {
                out.push_str(&format!("set -gx {k} {}\n", quote_fish(v)));
            }
            out.push_str(&format!("set -gx _BYPASS_GEN {gen}\n"));
            out.push_str(&format!("set -gx _BYPASS_KEYS {}\n", quote_fish(&keys_joined)));
        }
        Shell::Powershell => {
            for k in &to_unset {
                out.push_str(&format!(
                    "Remove-Item Env:\\{k} -ErrorAction SilentlyContinue\n"
                ));
            }
            for (k, v) in vars {
                out.push_str(&format!("$env:{k} = {}\n", quote_ps(v)));
            }
            out.push_str(&format!("$env:_BYPASS_GEN = '{gen}'\n"));
            out.push_str(&format!("$env:_BYPASS_KEYS = {}\n", quote_ps(&keys_joined)));
        }
    }
    out
}

/// POSIX single-quote: wrap in `'...'`, turning each `'` into `'\''`.
fn quote_sh(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Fish single-quote: inside `'...'` only `\` and `'` are special.
fn quote_fish(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

/// PowerShell single-quote: inside `'...'` a literal `'` is doubled.
fn quote_ps(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Env var names we are willing to emit. Anything else is skipped so we never
/// produce broken shell code.
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .enumerate()
            .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit()))
}

fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sh_quoting_handles_specials() {
        assert_eq!(quote_sh("abc"), "'abc'");
        assert_eq!(quote_sh("a'b"), "'a'\\''b'");
        assert_eq!(quote_sh("a b$c`d"), "'a b$c`d'");
        assert_eq!(quote_sh("line1\nline2"), "'line1\nline2'");
    }

    #[test]
    fn fish_quoting_escapes_backslash_and_quote() {
        assert_eq!(quote_fish("a'b"), "'a\\'b'");
        assert_eq!(quote_fish("a\\b"), "'a\\\\b'");
    }

    #[test]
    fn ps_quoting_doubles_quote() {
        assert_eq!(quote_ps("a'b"), "'a''b'");
    }

    #[test]
    fn name_validation() {
        assert!(is_valid_name("FOO"));
        assert!(is_valid_name("_FOO_BAR2"));
        assert!(!is_valid_name("2FOO"));
        assert!(!is_valid_name("FO O"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("FOO-BAR"));
    }

    #[test]
    fn emit_unsets_removed_keys() {
        let vars = vec![("NEW".to_string(), "v".to_string())];
        let old = vec!["OLD".to_string(), "NEW".to_string()];
        let out = emit(Shell::Sh, 7, &vars, &old);
        assert!(out.contains("unset OLD\n"));
        assert!(!out.contains("unset NEW"));
        assert!(out.contains("export NEW='v'\n"));
        assert!(out.contains("export _BYPASS_GEN=7\n"));
        assert!(out.contains("export _BYPASS_KEYS='NEW'\n"));
    }
}
