//! Lint engine + config cascade tests.

use okftool_core::{
    build_bundle, check, lint, rule_descriptors, rule_metas, ResolvedConfig, Severity,
};

fn bundle_with(files: Vec<(&str, &str)>) -> okftool_core::Bundle {
    build_bundle(files)
}

fn bundle_with_owned(files: Vec<(&str, String)>) -> okftool_core::Bundle {
    build_bundle(files)
}

fn codes(diags: &[okftool_core::Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

fn clean_doc(body: &str) -> String {
    format!("---\ntype: Note\ndescription: d\ntimestamp: 2026-01-01\n---\n{body}\n")
}

fn clean_doc_with_frontmatter(extra: &str, body: &str) -> String {
    format!("---\ntype: Note\ndescription: d\ntimestamp: 2026-01-01\n{extra}---\n{body}\n")
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
    assert!(c.contains(&"frontmatter/require-description"), "{c:?}");
    assert!(c.contains(&"frontmatter/require-timestamp"), "{c:?}");
    assert!(c.contains(&"body/structural-body"), "{c:?}");
    // linking/no-dangling-links is off by default → never present
    assert!(!c.contains(&"linking/no-dangling-links"), "{c:?}");
    // lint never affects conformance
    assert!(bundle.conformant);
}

#[test]
fn diagnostics_use_canonical_rule_ids_and_include_rule_metadata() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose only\n")]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let desc = diags
        .iter()
        .find(|d| d.code == "frontmatter/require-description")
        .expect("description diagnostic");
    assert_eq!(desc.rule_name.as_deref(), Some("Require Description"));
    assert_eq!(desc.category.as_deref(), Some("frontmatter"));
    assert_eq!(desc.category_name.as_deref(), Some("Frontmatter"));
    assert!(desc.rationale.as_deref().is_some_and(|r| !r.is_empty()));
    assert!(desc.help.as_deref().is_some_and(|h| !h.is_empty()));
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
    assert!(codes(&diags).contains(&"type-vocabulary/consistent-type-casing"));
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
        .find(|d| d.code == "frontmatter/timestamp-iso8601")
        .expect("flagged");
    assert_eq!(d.severity, Severity::Error);
    assert!(!d.spec); // it's a lint rule, not a spec error
}

#[test]
fn config_severity_override_and_off() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose only\n")]);
    // bump frontmatter/require-description to error, silence body/structural-body
    let cfg = ResolvedConfig::from_yaml(
        "rules:\n  frontmatter/require-description: error\n  body/structural-body: \"off\"\n  frontmatter/require-timestamp: \"off\"\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    let desc = diags
        .iter()
        .find(|d| d.code == "frontmatter/require-description")
        .unwrap();
    assert_eq!(desc.severity, Severity::Error);
    assert!(!codes(&diags).contains(&"body/structural-body"));
}

#[test]
fn flat_rule_ids_remain_supported_as_config_aliases() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose only\n")]);
    let cfg = ResolvedConfig::from_yaml(
        "rules:\n  require-description: error\n  structural-body: \"off\"\n  require-timestamp: \"off\"\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    let desc = diags
        .iter()
        .find(|d| d.code == "frontmatter/require-description")
        .unwrap();
    assert_eq!(desc.severity, Severity::Error);
    assert!(!codes(&diags).contains(&"body/structural-body"));
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
        "---\ntype: Note\nokf-lint-disable: [frontmatter/require-description, body/structural-body]\n---\nprose\n",
    )]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(!codes(&diags).contains(&"frontmatter/require-description"));
    assert!(!codes(&diags).contains(&"body/structural-body"));
}

#[test]
fn flat_rule_ids_remain_supported_as_inline_disable_aliases() {
    let bundle = bundle_with(vec![(
        "a.md",
        "---\ntype: Note\nokf-lint-disable: [require-description, structural-body]\n---\nprose\n",
    )]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(!codes(&diags).contains(&"frontmatter/require-description"));
    assert!(!codes(&diags).contains(&"body/structural-body"));
}

#[test]
fn extends_minimal_silences_most() {
    let bundle = bundle_with(vec![("a.md", "---\ntype: Note\n---\nprose\n")]);
    let cfg = ResolvedConfig::from_yaml("extends: okf-minimal\n").unwrap();
    let diags = lint(&bundle, &cfg);
    // minimal turns description/timestamp/structural off
    assert!(!codes(&diags).contains(&"frontmatter/require-description"));
    assert!(!codes(&diags).contains(&"body/structural-body"));
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
        assert!(m.id.contains('/'), "{} is not namespaced", m.id);
        assert!(!m.summary.is_empty() && !m.rationale.is_empty());
    }
}

#[test]
fn rule_descriptors_are_rich_enough_for_okfview() {
    let descriptors = rule_descriptors();
    let desc = descriptors
        .iter()
        .find(|d| d.id == "frontmatter/require-description")
        .expect("descriptor");
    assert_eq!(desc.slug, "require-description");
    assert_eq!(desc.name, "Require Description");
    assert_eq!(desc.aliases, vec!["require-description"]);
    assert_eq!(desc.category.id, "frontmatter");
    assert_eq!(desc.category.name, "Frontmatter");
    assert!(!desc.category.description.is_empty());
    assert_eq!(desc.default_severity, "warn");
    assert!(desc.docs_path.ends_with("require-description.md"));
}

#[test]
fn graph_structure_flags_excessive_bridging_ratio_and_leaf_fanout() {
    let source = clean_doc(
        "# Source\n- [local](/a/local.md)\n- This bridge connects the topic to [b](/b/one.md) because the same model appears in another area.\n- This bridge connects the topic to [c](/c/one.md) because the same model appears in another area.\n- This bridge connects the topic to [d](/d/one.md) because the same model appears in another area.\n",
    );
    let bundle = bundle_with_owned(vec![
        ("a/source.md", source),
        (
            "a/local.md",
            clean_doc("# Local\n- [source](/a/source.md)\n"),
        ),
        ("b/one.md", clean_doc("# B\n- [source](/a/source.md)\n")),
        ("c/one.md", clean_doc("# C\n- [source](/a/source.md)\n")),
        ("d/one.md", clean_doc("# D\n- [source](/a/source.md)\n")),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let source_codes: Vec<&str> = diags
        .iter()
        .filter(|d| d.file == "a/source.md")
        .map(|d| d.code.as_str())
        .collect();
    assert!(
        source_codes.contains(&"graph-structure/no-excessive-bridging"),
        "{source_codes:?}"
    );
    assert!(
        source_codes.contains(&"graph-structure/bridging-ratio"),
        "{source_codes:?}"
    );
    assert!(
        source_codes.contains(&"graph-structure/no-leaf-bridge-fanout"),
        "{source_codes:?}"
    );
}

#[test]
fn graph_structure_requires_local_cohesion_for_non_hubs() {
    let source = clean_doc(
        "# Source\n- This bridge connects the topic to [b](/b/one.md) because the same model appears in another area.\n- This bridge connects the topic to [c](/c/one.md) because the same model appears in another area.\n",
    );
    let bundle = bundle_with_owned(vec![
        ("a/source.md", source),
        ("b/one.md", clean_doc("# B\n- [source](/a/source.md)\n")),
        ("c/one.md", clean_doc("# C\n- [source](/a/source.md)\n")),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(
        diags
            .iter()
            .any(|d| { d.file == "a/source.md" && d.code == "graph-structure/min-local-cohesion" }),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_requires_prose_around_bridge_links() {
    let bundle = bundle_with_owned(vec![
        ("a/source.md", clean_doc("# Source\n- [b](/b/one.md)\n")),
        ("b/one.md", clean_doc("# B\n- [source](/a/source.md)\n")),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(
        diags.iter().any(|d| {
            d.file == "a/source.md" && d.code == "graph-structure/require-bridge-prose"
        }),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_prefers_neighborhood_index_over_many_deep_links() {
    let source = clean_doc(
        "# Source\n- This bridge connects the topic to [one](/b/one.md) because the same model appears in another area.\n- This bridge connects the topic to [two](/b/two.md) because the same model appears in another area.\n- This bridge connects the topic to [three](/b/three.md) because the same model appears in another area.\n",
    );
    let bundle = bundle_with_owned(vec![
        ("a/source.md", source),
        ("b/one.md", clean_doc("# One\n- [source](/a/source.md)\n")),
        ("b/two.md", clean_doc("# Two\n- [source](/a/source.md)\n")),
        (
            "b/three.md",
            clean_doc("# Three\n- [source](/a/source.md)\n"),
        ),
    ]);
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(
        diags.iter().any(|d| {
            d.file == "a/source.md" && d.code == "graph-structure/prefer-neighborhood-index-link"
        }),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_allows_configured_neighborhoods_to_override_inference() {
    let source = clean_doc_with_frontmatter(
        "neighborhood: source-frontmatter\n",
        "# Source\n- [target](/b/target.md)\n",
    );
    let target = clean_doc_with_frontmatter(
        "neighborhood: target-frontmatter\n",
        "# Target\n- [source](/a/source.md)\n",
    );
    let bundle = bundle_with_owned(vec![("a/source.md", source), ("b/target.md", target)]);
    let cfg = ResolvedConfig::from_yaml(
        "graph:\n  neighborhoods:\n    shared:\n      paths:\n        - a/source.md\n        - b/target.md\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    assert!(
        diags
            .iter()
            .all(|d| !d.code.starts_with("graph-structure/")),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_neighborhood_config_accepts_globs() {
    let bundle = bundle_with_owned(vec![
        (
            "reference/rules/source.md",
            clean_doc("# Source\n- [target](/reference/validation.md)\n"),
        ),
        (
            "reference/validation.md",
            clean_doc("# Validation\n- [source](/reference/rules/source.md)\n"),
        ),
    ]);
    let cfg = ResolvedConfig::from_yaml(
        "graph:\n  neighborhoods:\n    reference:\n      paths:\n        - reference/*.md\n        - reference/rules/*.md\n",
    )
    .unwrap();
    let diags = lint(&bundle, &cfg);
    assert!(
        diags
            .iter()
            .all(|d| !d.code.starts_with("graph-structure/")),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_detects_dense_neighborhood_cliques() {
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 1..=5 {
        let mut body = format!("# Node {i}\n");
        for j in 1..=5 {
            if i != j {
                body.push_str(&format!("- [node {j}](/a/node{j}.md)\n"));
            }
        }
        files.push((format!("a/node{i}.md"), clean_doc(&body)));
    }
    let bundle = bundle_with(
        files
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect(),
    );
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(
        diags
            .iter()
            .any(|d| d.code == "graph-structure/no-complete-neighborhood-clique"),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_asks_high_fanout_concepts_to_declare_hubs() {
    let mut body = "# Source\n".to_string();
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 1..=8 {
        body.push_str(&format!("- [target {i}](/n{i}/target.md)\n"));
        files.push((
            format!("n{i}/target.md"),
            clean_doc(&format!("# Target {i}\n- [source](/a/source.md)\n")),
        ));
    }
    files.push(("a/source.md".to_string(), clean_doc(&body)));
    let bundle = bundle_with(
        files
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect(),
    );
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    assert!(
        diags
            .iter()
            .any(|d| { d.file == "a/source.md" && d.code == "graph-structure/declare-hubs" }),
        "{diags:#?}"
    );
}

#[test]
fn graph_structure_hub_true_exempts_leaf_shape_rules() {
    let mut body = "# Source\n".to_string();
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 1..=4 {
        body.push_str(&format!("- [target {i}](/n{i}/target.md)\n"));
        files.push((
            format!("n{i}/target.md"),
            clean_doc(&format!("# Target {i}\n- [source](/a/source.md)\n")),
        ));
    }
    files.push((
        "a/source.md".to_string(),
        clean_doc_with_frontmatter("hub: true\n", &body),
    ));
    let bundle = bundle_with(
        files
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect(),
    );
    let diags = lint(&bundle, &ResolvedConfig::recommended());
    let source_codes: Vec<&str> = diags
        .iter()
        .filter(|d| d.file == "a/source.md")
        .map(|d| d.code.as_str())
        .collect();
    assert!(
        !source_codes.contains(&"graph-structure/no-excessive-bridging"),
        "{source_codes:?}"
    );
    assert!(
        !source_codes.contains(&"graph-structure/bridging-ratio"),
        "{source_codes:?}"
    );
    assert!(
        !source_codes.contains(&"graph-structure/no-leaf-bridge-fanout"),
        "{source_codes:?}"
    );
    assert!(
        !source_codes.contains(&"graph-structure/min-local-cohesion"),
        "{source_codes:?}"
    );
}
