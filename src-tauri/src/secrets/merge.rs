//! Merging the variables of several active contexts into the single set that
//! shells receive.
//!
//! Contexts are layered by priority: the first context to define a key owns
//! it, and the same key in a lower-priority context is shadowed. `{$VAR}`
//! references are resolved against the merged set, so a variable in one
//! context may reference one defined by another.

use serde::Serialize;
use std::collections::BTreeMap;

use super::interpolate::resolve_all;

/// A variable of the merged active set.
#[derive(Debug, Clone, Serialize)]
pub struct MergedVar {
    pub key: String,
    /// The template as stored in `source`.
    pub raw: String,
    /// The value after interpolation against the merged set.
    pub value: String,
    /// Context that owns the key (highest-priority one defining it).
    pub source: String,
    /// Lower-priority contexts that also define the key, in priority order.
    pub shadowed: Vec<String>,
    /// Set when a reference is missing, cyclic or too deep.
    pub issue: Option<String>,
}

/// Layers `contexts` (highest priority first, each with its raw variables)
/// into one resolved set. Output is sorted by key.
pub fn merge(contexts: &[(String, BTreeMap<String, String>)]) -> Vec<MergedVar> {
    let mut templates: BTreeMap<String, String> = BTreeMap::new();
    let mut sources: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();

    for (name, vars) in contexts {
        for (key, raw) in vars {
            match sources.get_mut(key) {
                Some((_, shadowed)) => shadowed.push(name.clone()),
                None => {
                    templates.insert(key.clone(), raw.clone());
                    sources.insert(key.clone(), (name.clone(), Vec::new()));
                }
            }
        }
    }

    resolve_all(&templates)
        .into_iter()
        .map(|r| {
            let (source, shadowed) = sources
                .remove(&r.key)
                .expect("every template has a source");
            MergedVar {
                key: r.key,
                raw: r.raw,
                value: r.value,
                source,
                shadowed,
                issue: r.issue,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str, pairs: &[(&str, &str)]) -> (String, BTreeMap<String, String>) {
        (
            name.to_string(),
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }

    fn find<'a>(all: &'a [MergedVar], key: &str) -> &'a MergedVar {
        all.iter().find(|m| m.key == key).expect("key present")
    }

    #[test]
    fn first_context_wins_and_shadows_the_rest() {
        let all = merge(&[
            ctx("prod", &[("TOKEN", "p")]),
            ctx("dev", &[("TOKEN", "d"), ("ONLY_DEV", "1")]),
            ctx("base", &[("TOKEN", "b")]),
        ]);
        let token = find(&all, "TOKEN");
        assert_eq!(token.value, "p");
        assert_eq!(token.source, "prod");
        assert_eq!(token.shadowed, vec!["dev", "base"]);
        assert_eq!(find(&all, "ONLY_DEV").source, "dev");
    }

    #[test]
    fn references_resolve_across_contexts() {
        let all = merge(&[
            ctx("app", &[("URL", "https://{$HOST}/api")]),
            ctx("infra", &[("HOST", "example.com")]),
        ]);
        let url = find(&all, "URL");
        assert_eq!(url.value, "https://example.com/api");
        assert!(url.issue.is_none());
    }

    #[test]
    fn reference_follows_the_winning_definition() {
        let all = merge(&[
            ctx("override", &[("HOST", "staging.example.com")]),
            ctx("app", &[("HOST", "example.com"), ("URL", "https://{$HOST}/")]),
        ]);
        assert_eq!(find(&all, "URL").value, "https://staging.example.com/");
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(merge(&[]).is_empty());
    }
}
