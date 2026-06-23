//! Bundle assembly: parse every file, resolve the cross-link graph, and collect
//! spec (§9) conformance diagnostics. Fully permissive — never rejects a bundle.
//!
//! Mirrors okfview's `buildBundle`, except `broken-link` / `index-frontmatter`
//! are *not* emitted here: those are advisory and belong to the (disableable)
//! lint layer. `Link::broken` is still resolved so lint rules can report it.

use std::collections::HashSet;

use crate::links::resolve_target;
use crate::model::{Bundle, Severity};
use crate::parse::{parse_concept, parse_index, parse_log, reserved_kind, Reserved};

/// Build a [`Bundle`] from `(path, content)` files. `path` is bundle-relative;
/// only `.md` files are considered.
pub fn build_bundle<I, P, C>(files: I) -> Bundle
where
    I: IntoIterator<Item = (P, C)>,
    P: AsRef<str>,
    C: AsRef<str>,
{
    let mut concepts = Vec::new();
    let mut indexes = Vec::new();
    let mut logs = Vec::new();
    let mut diagnostics = Vec::new();
    let mut okf_version = None;

    for (path, content) in files {
        let path = path.as_ref();
        if !path.to_lowercase().ends_with(".md") {
            continue;
        }
        match reserved_kind(path) {
            Some(Reserved::Index) => {
                let parsed = parse_index(path, content.as_ref());
                if parsed.index.dir.is_empty() {
                    if let Some(v) = &parsed.okf_version {
                        okf_version = Some(v.clone());
                    }
                }
                indexes.push(parsed.index);
            }
            Some(Reserved::Log) => logs.push(parse_log(path, content.as_ref())),
            None => {
                let (concept, diags) = parse_concept(path, content.as_ref());
                concepts.push(concept);
                diagnostics.extend(diags);
            }
        }
    }

    // Resolve cross-links now that every concept id is known.
    let id_set: HashSet<String> = concepts.iter().map(|c| c.id.clone()).collect();
    for concept in &mut concepts {
        let from = concept.id.clone();
        for link in &mut concept.outgoing {
            let resolved = resolve_target(&from, &link.href);
            if let Some(ext) = resolved.external {
                link.external = Some(ext);
            } else if resolved.directory_or_anchor {
                // directory / index / anchor links are never broken
            } else if let Some(target) = resolved.target_path {
                if id_set.contains(&target) {
                    link.target_id = Some(target);
                } else {
                    link.broken = true;
                }
            }
        }
    }

    let mut types: Vec<String> = concepts
        .iter()
        .filter_map(|c| c.concept_type.clone())
        .collect();
    types.sort();
    types.dedup();

    let conformant = !diagnostics
        .iter()
        .any(|d| d.spec && d.severity == Severity::Error);

    Bundle {
        concepts,
        indexes,
        logs,
        types,
        okf_version,
        diagnostics,
        conformant,
    }
}
