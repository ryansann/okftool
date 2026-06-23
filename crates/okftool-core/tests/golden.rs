//! Golden test over the committed `testdata/sample-bundle` corpus: the curated
//! bundle must trigger exactly the expected rule set (and stay silent elsewhere).
//! This is the CI-safe counterpart to the gitignored `fixtures/` parity tests.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use okftool_core::{build_bundle, lint, ResolvedConfig, Severity};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cases/all-rules")
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

fn corpus() -> Vec<(String, String)> {
    read_md(&corpus_dir())
}

#[test]
fn recommended_profile_triggers_exact_rule_set() {
    let bundle = build_bundle(corpus());
    assert!(
        bundle.conformant,
        "sample bundle must be conformant: {:?}",
        bundle.diagnostics
    );

    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let got: BTreeSet<&str> = diags.iter().map(|d| d.code.as_str()).collect();
    let want: BTreeSet<&str> = [
        "consistent-type-casing",
        "index-entry-has-description",
        "no-orphan-concepts",
        "no-singleton-type",
        "no-unindexed-concepts",
        "prefer-absolute-links",
        "require-description",
        "require-timestamp",
        "structural-body",
        "timestamp-iso8601",
    ]
    .into_iter()
    .collect();
    assert_eq!(got, want, "unexpected rule set under recommended");

    // The error-level rule is present and actually an error.
    assert!(diags
        .iter()
        .any(|d| d.code == "timestamp-iso8601" && d.severity == Severity::Error));
    // Off-by-default / unmet rules stay absent.
    assert!(!got.contains("no-dangling-links"));
    assert!(!got.contains("max-out-degree"));
}

#[test]
fn strict_profile_enables_dangling_links() {
    let bundle = build_bundle(corpus());
    let cfg = ResolvedConfig::from_yaml("extends: okf-strict\n").unwrap();
    let diags = lint(&bundle, &cfg);
    assert!(
        diags
            .iter()
            .any(|d| d.code == "no-dangling-links" && d.file == "concepts/messy.md"),
        "okf-strict should flag the broken link in messy.md"
    );
}

#[test]
fn max_out_degree_fires_with_a_low_cap() {
    let bundle = build_bundle(corpus());
    let cfg = ResolvedConfig::from_yaml(
        "rules:\n  max-out-degree:\n    severity: warn\n    options: { max: 2 }\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    assert!(
        diags
            .iter()
            .any(|d| d.code == "max-out-degree" && d.file == "concepts/overview.md"),
        "overview links out to 3 concepts and should exceed max: 2"
    );
}
