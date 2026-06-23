//! Lint engine + config cascade tests.

use okftool_core::{build_bundle, check, lint, rule_metas, ResolvedConfig, Severity};

fn bundle_with(files: Vec<(&str, &str)>) -> okftool_core::Bundle {
    build_bundle(files)
}

fn codes(diags: &[okftool_core::Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn recommended_flags_expected_rules() {
    let bundle = bundle_with(vec![
        // orphan, no description, no timestamp, prose-only body
        ("a.md", "---\ntype: Note\n---\nJust a wall of prose with no structure.\n"),
        // links to b (so a/b not orphan if linked); b has description + structure
        (
            "hub.md",
            "---\ntype: Note\ndescription: A hub.\ntimestamp: 2026-01-01\n---\n# Hub\n- [a](/a.md)\n",
        ),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let c = codes(&diags);
    assert!(c.contains(&"require-description"), "{c:?}");
    assert!(c.contains(&"require-timestamp"), "{c:?}");
    assert!(c.contains(&"structural-body"), "{c:?}");
    // no-dangling-links is off by default → never present
    assert!(!c.contains(&"no-dangling-links"), "{c:?}");
    // lint never affects conformance
    assert!(bundle.conformant);
}

#[test]
fn consistent_type_casing_detects_variants() {
    let bundle = bundle_with(vec![
        (
            "x.md",
            "---\ntype: Table\ndescription: d\ntimestamp: 2026-01-01\n---\n# x\n- [y](/y.md)\n",
        ),
        (
            "y.md",
            "---\ntype: table\ndescription: d\ntimestamp: 2026-01-01\n---\n# y\n- [x](/x.md)\n",
        ),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(codes(&diags).contains(&"consistent-type-casing"));
}

#[test]
fn timestamp_iso8601_is_error_and_flags_bad_format() {
    let bundle = bundle_with(vec![(
        "x.md",
        "---\ntype: Note\ndescription: d\ntimestamp: 05/22/2026\n---\n# x\n- [x](/x.md)\n",
    )]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let d = diags
        .iter()
        .find(|d| d.code == "timestamp-iso8601")
        .expect("flagged");
    assert_eq!(d.severity, Severity::Error);
    assert!(!d.spec); // it's a lint rule, not a spec error
}

#[test]
fn config_severity_override_and_off() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose only\n")]);
    // bump require-description to error, silence structural-body
    let cfg = ResolvedConfig::from_yaml(
        "rules:\n  require-description: error\n  structural-body: \"off\"\n  require-timestamp: \"off\"\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    let desc = diags
        .iter()
        .find(|d| d.code == "require-description")
        .unwrap();
    assert_eq!(desc.severity, Severity::Error);
    assert!(!codes(&diags).contains(&"structural-body"));
}

#[test]
fn path_overrides_silence_a_subtree() {
    let bundle = bundle_with(vec![
        ("drafts/wip.md", "---\ntype: Note\n---\nprose\n"),
        ("real.md", "---\ntype: Note\n---\nprose\n"),
    ]);
    let cfg = ResolvedConfig::from_yaml(
        "overrides:\n  - files: \"drafts/**\"\n    rules:\n      \"*\": \"off\"\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    assert!(
        diags.iter().all(|d| !d.file.starts_with("drafts/")),
        "drafts silenced"
    );
    assert!(
        diags.iter().any(|d| d.file == "real.md"),
        "real.md still linted"
    );
}

#[test]
fn inline_disable_in_frontmatter() {
    let bundle = bundle_with(vec![(
        "a.md",
        "---\ntype: Note\nokf-lint-disable: [require-description, structural-body]\n---\nprose\n",
    )]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(!codes(&diags).contains(&"require-description"));
    assert!(!codes(&diags).contains(&"structural-body"));
}

#[test]
fn extends_minimal_silences_most() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose\n")]);
    let cfg = ResolvedConfig::from_yaml("extends: okf-minimal\n").unwrap();
    let diags = lint(&bundle, &cfg);
    // minimal turns description/timestamp/structural off
    assert!(!codes(&diags).contains(&"require-description"));
    assert!(!codes(&diags).contains(&"structural-body"));
}

#[test]
fn check_merges_spec_and_lint_but_conformant_is_spec_only() {
    // missing type (spec error) + lint findings together
    let bundle = check(
        vec![("a.md", "---\ntitle: x\n---\nprose\n")],
        &ResolvedConfig::recommended(),
    );
    assert!(!bundle.conformant);
    assert!(bundle
        .diagnostics
        .iter()
        .any(|d| d.code == "missing-type" && d.spec));
    assert!(bundle.diagnostics.iter().any(|d| !d.spec)); // lint findings present
}

#[test]
fn every_rule_has_unique_id_and_metadata() {
    let metas = rule_metas();
    assert!(metas.len() >= 9);
    let mut ids: Vec<&str> = metas.iter().map(|m| m.id).collect();
    ids.sort();
    let unique = ids.len();
    ids.dedup();
    assert_eq!(unique, ids.len(), "rule ids must be unique");
    for m in &metas {
        assert!(!m.summary.is_empty() && !m.rationale.is_empty());
    }
}
