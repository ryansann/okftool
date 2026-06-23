//! Unit + integration tests for parsing and §9 validation. Cases mirror
//! okfview's `core.test.ts` so the two implementations stay in lock-step.

use okflint_core::{build_bundle, parse_concept, parse_index, parse_log, resolve_target};

#[test]
fn requires_only_type_and_preserves_unknown_keys() {
    let (concept, diags) = parse_concept(
        "a/b.md",
        "---\ntype: Metric\ncustom_key: hello\n---\n# Body\n",
    );
    assert_eq!(concept.concept_type.as_deref(), Some("Metric"));
    assert_eq!(
        concept
            .frontmatter
            .get("custom_key")
            .and_then(|v| v.as_str()),
        Some("hello")
    );
    assert!(diags.is_empty());
}

#[test]
fn flags_missing_type_but_still_produces_concept() {
    let (concept, diags) = parse_concept("x.md", "---\ntitle: No type\n---\nbody");
    assert_eq!(concept.concept_type, None);
    assert!(diags.iter().any(|d| d.code == "missing-type"));
}

#[test]
fn flags_missing_frontmatter() {
    let (_concept, diags) = parse_concept("x.md", "# Just markdown\n");
    assert!(diags.iter().any(|d| d.code == "missing-frontmatter"));
}

#[test]
fn resolves_absolute_and_relative_links_and_ignores_index_anchors() {
    assert_eq!(
        resolve_target("tables/orders", "/tables/customers.md")
            .target_path
            .as_deref(),
        Some("tables/customers")
    );
    assert_eq!(
        resolve_target("a/b/c", "../d.md").target_path.as_deref(),
        Some("a/d")
    );
    assert_eq!(
        resolve_target("a/b", "https://x.com").external.as_deref(),
        Some("https://x.com")
    );
    assert!(resolve_target("a/b", "/datasets/index.md").directory_or_anchor);
    assert!(resolve_target("a/b", "#section").directory_or_anchor);
}

#[test]
fn reads_okf_version_only_from_root_index() {
    let root = parse_index("index.md", "---\nokf_version: \"0.1\"\n---\n# Sub\n");
    assert_eq!(root.okf_version.as_deref(), Some("0.1"));
    assert!(root.index.dir.is_empty());

    let nested = parse_index("sub/index.md", "---\nokf_version: \"0.1\"\n---\n");
    assert!(nested.index.has_frontmatter);
    assert_eq!(nested.index.dir, "sub");
}

#[test]
fn parses_log_days_and_verbs() {
    let log = parse_log(
        "log.md",
        "# Log\n\n## 2026-05-22\n* **Update**: did a thing.\n* plain entry\n",
    );
    assert_eq!(log.days[0].date, "2026-05-22");
    assert_eq!(log.days[0].entries[0].verb.as_deref(), Some("Update"));
    assert_eq!(log.days[0].entries[1].text, "plain entry");
}

#[test]
fn build_bundle_verdict_and_link_resolution() {
    let files = vec![
        ("index.md", "# Bundle\n- [Thing](/concepts/thing.md)\n"),
        (
            "concepts/thing.md",
            "---\ntype: Concept\n---\nlinks to [other](/concepts/other.md) and [self](/concepts/thing.md)\n",
        ),
        ("concepts/other.md", "---\ntype: Concept\n---\nbody\n"),
        ("concepts/no-type.md", "---\ntitle: No Type\n---\nbody\n"),
    ];
    let bundle = build_bundle(files);

    // index.md excluded from concepts.
    assert_eq!(bundle.concepts.len(), 3);
    assert_eq!(bundle.types, vec!["Concept".to_string()]);

    // Only the §9 error fires; conformance reflects it.
    assert!(!bundle.conformant);
    assert!(bundle.diagnostics.iter().any(|d| d.code == "missing-type"));
    assert!(bundle.diagnostics.iter().all(|d| d.spec));

    // Link resolution: /concepts/other resolves, /concepts/thing resolves (self).
    let thing = bundle
        .concepts
        .iter()
        .find(|c| c.id == "concepts/thing")
        .unwrap();
    assert!(thing
        .outgoing
        .iter()
        .any(|l| l.target_id.as_deref() == Some("concepts/other")));
    assert!(thing.outgoing.iter().all(|l| !l.broken));
}

#[test]
fn unknown_type_and_broken_links_are_not_spec_errors() {
    // Per §9 these MUST NOT be rejected.
    let bundle = build_bundle(vec![(
        "concepts/x.md",
        "---\ntype: SomethingNovel\n---\nlink to [missing](/nope.md)\n",
    )]);
    assert!(bundle.conformant);
    // The broken link is recorded (for lint) but raises no spec diagnostic.
    let x = &bundle.concepts[0];
    assert!(x.outgoing.iter().any(|l| l.broken));
    assert!(bundle.diagnostics.is_empty());
}
