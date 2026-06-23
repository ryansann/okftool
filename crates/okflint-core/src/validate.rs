//! Spec (§9) conformance validation.
//!
//! A bundle is conformant iff every non-reserved `.md` file has a parseable
//! frontmatter block with a non-empty `type`. Per spec, consumers MUST NOT
//! reject for missing optional fields, unknown `type` values, extra keys, broken
//! cross-links, or missing `index.md` files — so none of those are errors here;
//! they belong to the (disableable) lint layer.

use crate::model::{Bundle, Severity};
use crate::parse::{is_reserved, parse_concept};

/// Validate a set of `(path, content)` files. `path` is bundle-relative; only
/// `.md` files are considered (callers should pre-filter, but non-`.md` paths
/// are ignored defensively).
pub fn validate_files<I, P, C>(files: I) -> Bundle
where
    I: IntoIterator<Item = (P, C)>,
    P: AsRef<str>,
    C: AsRef<str>,
{
    let mut concepts = Vec::new();
    let mut diagnostics = Vec::new();

    for (path, content) in files {
        let path = path.as_ref();
        if !path.ends_with(".md") {
            continue;
        }
        let (concept, diags) = parse_concept(path, content.as_ref());
        diagnostics.extend(diags);
        if !is_reserved(path) {
            concepts.push(concept);
        }
    }

    let conformant = !diagnostics
        .iter()
        .any(|d| d.spec && d.severity == Severity::Error);

    Bundle {
        concepts,
        diagnostics,
        conformant,
    }
}
