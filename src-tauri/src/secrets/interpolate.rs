//! `{$VAR}` interpolation between variables of the same credential context.
//!
//! A value may reference another variable of its own context, so
//! `APP_PATH_CONFIG = {$APP_PATH}/config` resolves to the current value of
//! `APP_PATH` followed by `/config`. References are resolved lazily, every time
//! the variables are handed to a shell: the vault always stores the template, so
//! editing `APP_PATH` updates everything derived from it.
//!
//! The trigger is the two-character sequence `{$`, never a bare `$`. Secrets
//! that contain dollar signs (bcrypt hashes, passwords) therefore need no
//! escaping at all. A literal `{$` is written `\{$`.
//!
//! Anything that cannot be resolved — unknown name, missing `}`, malformed name,
//! reference cycle — is left in the output verbatim and reported through
//! [`ResolvedVar::issue`], so a typo surfaces in the UI instead of silently
//! exporting an empty value.

use serde::Serialize;
use std::collections::BTreeMap;

/// Nesting limit for chained references. Cycles are caught separately; this only
/// bounds pathological depth.
const MAX_DEPTH: usize = 32;

/// Output size limit per variable. Guards against fan-out blow-up, where each
/// level references the one below it twice.
const MAX_LEN: usize = 1 << 20;

/// A variable with its references resolved.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedVar {
    pub key: String,
    /// The template exactly as stored. This is what the editor shows and saves;
    /// resolving on write would freeze the reference.
    pub raw: String,
    /// The value after interpolation. Unresolvable references are left as-is.
    pub value: String,
    /// Human-readable description of the first problem found, if any.
    pub issue: Option<String>,
}

/// Resolves every variable of a context against the others. Key order is
/// preserved from the input map; resolution itself is order-independent because
/// references are followed recursively.
pub fn resolve_all(vars: &BTreeMap<String, String>) -> Vec<ResolvedVar> {
    vars.iter()
        .map(|(key, raw)| {
            let mut issue = None;
            let mut stack = vec![key.as_str()];
            let value = expand(raw, vars, &mut stack, &mut issue);
            ResolvedVar {
                key: key.clone(),
                raw: raw.clone(),
                value,
                issue,
            }
        })
        .collect()
}

/// Expands references in `raw`. `stack` holds the chain of keys currently being
/// expanded, innermost last, and is what makes cycle detection possible.
fn expand<'a>(
    raw: &str,
    vars: &'a BTreeMap<String, String>,
    stack: &mut Vec<&'a str>,
    issue: &mut Option<String>,
) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;

    while i < raw.len() {
        let rest = &raw[i..];

        // `\{$` is a literal `{$`.
        if rest.starts_with("\\{$") {
            out.push_str("{$");
            i += 3;
            continue;
        }

        if let Some((name, consumed)) = parse_ref(rest) {
            i += consumed;
            out.push_str(&expand_ref(name, vars, stack, issue));
            if out.len() > MAX_LEN {
                record(issue, format!("value of {} is too large to expand", stack[0]));
                out.truncate(MAX_LEN);
                return out;
            }
            continue;
        }

        // Not a reference: copy one character verbatim.
        let ch = rest.chars().next().expect("i < raw.len() implies a char");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

/// Resolves a single `{$name}` occurrence to its replacement text.
fn expand_ref<'a>(
    name: &str,
    vars: &'a BTreeMap<String, String>,
    stack: &mut Vec<&'a str>,
    issue: &mut Option<String>,
) -> String {
    let literal = format!("{{${name}}}");

    if let Some(pos) = stack.iter().position(|k| *k == name) {
        let mut chain: Vec<&str> = stack[pos..].to_vec();
        chain.push(name);
        record(issue, format!("reference cycle: {}", chain.join(" → ")));
        return literal;
    }

    if stack.len() >= MAX_DEPTH {
        record(issue, format!("{name} is nested too deeply"));
        return literal;
    }

    // Borrow the key from the map, not from `name`, so it outlives this frame.
    let Some((key, raw)) = vars.get_key_value(name) else {
        record(issue, format!("{name} is not defined in this context"));
        return literal;
    };

    stack.push(key.as_str());
    let expanded = expand(raw, vars, stack, issue);
    stack.pop();
    expanded
}

/// If `rest` starts with a well-formed `{$NAME}`, returns the name and how many
/// bytes it occupies. Malformed references are not matched, so they end up in
/// the output verbatim.
fn parse_ref(rest: &str) -> Option<(&str, usize)> {
    let after = rest.strip_prefix("{$")?;
    let close = after.find('}')?;
    let name = &after[..close];
    if !is_valid_name(name) {
        return None;
    }
    // `{$` + name + `}`
    Some((name, 2 + close + 1))
}

/// Environment-variable name rules. Kept in sync with `is_valid_name` in
/// `crates/bypass-shell/src/main.rs`, which drops anything else on the way out.
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .enumerate()
            .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit()))
}

/// Keeps the first issue found; later ones are usually consequences of it.
fn record(slot: &mut Option<String>, message: String) {
    slot.get_or_insert(message);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn resolve(pairs: &[(&str, &str)], key: &str) -> ResolvedVar {
        resolve_all(&map(pairs))
            .into_iter()
            .find(|r| r.key == key)
            .expect("key present")
    }

    #[test]
    fn expands_a_single_reference() {
        let r = resolve(&[("APP_PATH", "/opt/app"), ("CFG", "{$APP_PATH}/config")], "CFG");
        assert_eq!(r.value, "/opt/app/config");
        assert!(r.issue.is_none());
    }

    #[test]
    fn expands_chained_references_regardless_of_order() {
        // C depends on B depends on A, and C sorts before both.
        let r = resolve(
            &[("C", "{$B}/c"), ("B", "{$A}/b"), ("A", "/a")],
            "C",
        );
        assert_eq!(r.value, "/a/b/c");
        assert!(r.issue.is_none());
    }

    #[test]
    fn expands_several_references_in_one_value() {
        let r = resolve(
            &[("H", "example.com"), ("P", "8080"), ("URL", "https://{$H}:{$P}/api")],
            "URL",
        );
        assert_eq!(r.value, "https://example.com:8080/api");
    }

    #[test]
    fn bare_dollar_is_never_touched() {
        let r = resolve(&[("PASS", "p4ss$word$2y$10$abc")], "PASS");
        assert_eq!(r.value, "p4ss$word$2y$10$abc");
        assert!(r.issue.is_none());
    }

    #[test]
    fn shell_syntax_is_not_expanded() {
        let r = resolve(&[("HOME", "/nope"), ("P", "$HOME/x:${HOME}/y")], "P");
        assert_eq!(r.value, "$HOME/x:${HOME}/y");
    }

    #[test]
    fn backslash_escapes_a_literal_reference() {
        let r = resolve(&[("A", "1"), ("T", "\\{$A} stays")], "T");
        assert_eq!(r.value, "{$A} stays");
        assert!(r.issue.is_none());
    }

    #[test]
    fn malformed_references_are_left_verbatim() {
        let r = resolve(&[("A", "1"), ("T", "{$unclosed and {$FOO-BAR} and {$}")], "T");
        assert_eq!(r.value, "{$unclosed and {$FOO-BAR} and {$}");
        assert!(r.issue.is_none());
    }

    #[test]
    fn missing_reference_is_reported_and_left_verbatim() {
        let r = resolve(&[("T", "{$NOPE}/x")], "T");
        assert_eq!(r.value, "{$NOPE}/x");
        assert_eq!(
            r.issue.as_deref(),
            Some("NOPE is not defined in this context")
        );
    }

    #[test]
    fn direct_cycle_is_caught() {
        let r = resolve(&[("A", "{$A}")], "A");
        assert_eq!(r.value, "{$A}");
        assert_eq!(r.issue.as_deref(), Some("reference cycle: A → A"));
    }

    #[test]
    fn indirect_cycle_is_caught() {
        let r = resolve(&[("A", "{$B}"), ("B", "{$A}")], "A");
        assert_eq!(r.issue.as_deref(), Some("reference cycle: A → B → A"));
    }

    #[test]
    fn a_cycle_elsewhere_does_not_break_other_vars() {
        let all = resolve_all(&map(&[("A", "{$B}"), ("B", "{$A}"), ("OK", "fine")]));
        let ok = all.iter().find(|r| r.key == "OK").unwrap();
        assert_eq!(ok.value, "fine");
        assert!(ok.issue.is_none());
    }

    #[test]
    fn multibyte_values_survive() {
        let r = resolve(&[("N", "café"), ("T", "{$N} ☕/x")], "T");
        assert_eq!(r.value, "café ☕/x");
    }

    #[test]
    fn deep_nesting_is_bounded() {
        // Longer than MAX_DEPTH: V0 -> V1 -> ... -> V40.
        let mut pairs: Vec<(String, String)> = (0..40)
            .map(|i| (format!("V{i}"), format!("{{$V{}}}", i + 1)))
            .collect();
        pairs.push(("V40".to_string(), "end".to_string()));
        let vars: BTreeMap<String, String> = pairs.into_iter().collect();

        let r = resolve_all(&vars)
            .into_iter()
            .find(|r| r.key == "V0")
            .unwrap();
        assert!(r.issue.unwrap().contains("nested too deeply"));
    }
}
