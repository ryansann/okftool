use okflint_core::validate_files;

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        // reserved: no frontmatter required
        ("index.md", "# Bundle\n- [Thing](/concepts/thing.md)\n"),
        // well-formed concept
        (
            "concepts/thing.md",
            "---\ntype: Concept\ntitle: Thing\ndescription: A thing.\n---\n# Thing\nLinks to [x](/concepts/x.md).\n",
        ),
        // missing type -> spec error
        ("concepts/no-type.md", "---\ntitle: No Type\n---\nBody.\n"),
        // no frontmatter at all -> spec error
        ("concepts/bare.md", "Just prose, no frontmatter.\n"),
    ]
}

#[test]
fn validates_spec_conformance() {
    let bundle = validate_files(files());

    // index.md is reserved and excluded from concepts.
    assert_eq!(bundle.concepts.len(), 3);

    // Two §9 errors: missing-type and missing-frontmatter.
    let codes: Vec<&str> = bundle.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"missing-type"));
    assert!(codes.contains(&"missing-frontmatter"));
    assert!(bundle.diagnostics.iter().all(|d| d.spec));
    assert!(!bundle.conformant);
}

#[test]
fn clean_bundle_is_conformant() {
    let bundle = validate_files(vec![(
        "concepts/thing.md",
        "---\ntype: Concept\n---\nbody\n",
    )]);
    assert!(bundle.conformant);
    assert_eq!(bundle.concepts[0].concept_type.as_deref(), Some("Concept"));
    assert!(bundle.diagnostics.is_empty());
}

#[test]
fn unknown_type_and_broken_links_are_not_spec_errors() {
    // Per §9 these MUST NOT be rejected.
    let bundle = validate_files(vec![(
        "concepts/x.md",
        "---\ntype: SomethingNovel\n---\nlink to [missing](/nope.md)\n",
    )]);
    assert!(bundle.conformant);
}
