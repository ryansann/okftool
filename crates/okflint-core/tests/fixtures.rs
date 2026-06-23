//! Parity corpus: the official OKF fixtures (copied from okfview) must validate
//! exactly as the app reports them — conformant, with a resolved link graph.

use std::fs;
use std::path::{Path, PathBuf};

use okflint_core::build_bundle;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/.vendor")
        .join(name)
}

/// Fixtures are gitignored (vendored locally), so skip when they're not present
/// (e.g. a fresh clone or CI without the corpus).
fn require(name: &str) -> Option<PathBuf> {
    let path = fixture(name);
    if path.exists() {
        Some(path)
    } else {
        eprintln!("skipping: fixture `{name}` not present");
        None
    }
}

fn read_md(root: &Path) -> Vec<(String, String)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn official_fixtures_are_conformant() {
    for name in ["ga4", "stackoverflow", "crypto_bitcoin"] {
        let Some(dir) = require(name) else { continue };
        let bundle = build_bundle(read_md(&dir));
        assert!(!bundle.concepts.is_empty(), "{name}: has concepts");
        assert!(
            bundle.conformant,
            "{name}: expected conformant, diagnostics: {:?}",
            bundle.diagnostics
        );
    }
}

#[test]
fn ga4_has_types_and_resolved_links() {
    let Some(dir) = require("ga4") else { return };
    let bundle = build_bundle(read_md(&dir));
    assert!(
        bundle.types.iter().any(|t| t == "BigQuery Table"),
        "ga4 types: {:?}",
        bundle.types
    );
    let resolved = bundle
        .concepts
        .iter()
        .flat_map(|c| &c.outgoing)
        .filter(|l| l.target_id.is_some())
        .count();
    assert!(resolved > 0, "ga4 should resolve some in-bundle links");
}
