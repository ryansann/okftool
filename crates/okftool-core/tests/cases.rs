//! Targeted fixture cases: each small bundle under `fixtures/cases/<name>/`
//! isolates one rule (or validation) violation and must produce exactly that.
//! These are committed and double as worked examples of each diagnostic.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use okftool_core::{check, ResolvedConfig};

fn case_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/cases")
        .join(name)
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

struct Case {
    name: &'static str,
    /// Config YAML (`""` = okf-recommended).
    config: &'static str,
    conformant: bool,
    /// The exact set of diagnostic codes expected.
    codes: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "conformant",
        config: "",
        conformant: true,
        codes: &[],
    },
    // spec cases, isolated under okf-minimal so only spec codes remain
    Case {
        name: "missing-type",
        config: "extends: okf-minimal",
        conformant: false,
        codes: &["missing-type"],
    },
    Case {
        name: "missing-frontmatter",
        config: "extends: okf-minimal",
        conformant: false,
        codes: &["missing-frontmatter", "missing-type"],
    },
    // lint cases under the recommended profile
    Case {
        name: "orphan-concept",
        config: "",
        conformant: true,
        codes: &["no-orphan-concepts"],
    },
    Case {
        name: "relative-links",
        config: "",
        conformant: true,
        codes: &["prefer-absolute-links"],
    },
    Case {
        name: "type-casing",
        config: "",
        conformant: true,
        // Widget vs widget also makes each type a singleton.
        codes: &["consistent-type-casing", "no-singleton-type"],
    },
    Case {
        name: "sparse-frontmatter",
        config: "",
        conformant: true,
        codes: &["require-description", "require-timestamp"],
    },
    Case {
        name: "bad-timestamp",
        config: "",
        conformant: true,
        codes: &["timestamp-iso8601"],
    },
    Case {
        name: "prose-wall",
        config: "",
        conformant: true,
        codes: &["structural-body"],
    },
    // cases needing a specific config to demonstrate
    Case {
        name: "dangling-link",
        config: "rules:\n  no-dangling-links: warn\n",
        conformant: true,
        codes: &["no-dangling-links"],
    },
    Case {
        name: "hub-overflow",
        config: "rules:\n  max-out-degree:\n    severity: warn\n    options: { max: 2 }\n",
        conformant: true,
        codes: &["max-out-degree"],
    },
    // additional rules
    Case { name: "self-link", config: "", conformant: true, codes: &["no-self-link"] },
    Case { name: "singleton-type", config: "", conformant: true, codes: &["no-singleton-type"] },
    Case { name: "empty-values", config: "", conformant: true, codes: &["no-empty-frontmatter-values"] },
    Case { name: "empty-body", config: "", conformant: true, codes: &["body-not-empty"] },
    Case { name: "unindexed", config: "", conformant: true, codes: &["no-unindexed-concepts"] },
    Case { name: "log-order", config: "", conformant: true, codes: &["log-newest-first"] },
    Case {
        name: "index-entry-no-desc",
        config: "",
        conformant: true,
        codes: &["index-entry-has-description"],
    },
    // off-by-default rules demonstrated by reusing a fixture with a config
    Case {
        name: "relative-links",
        config: "rules:\n  no-relative-links: warn\n",
        conformant: true,
        codes: &["prefer-absolute-links", "no-relative-links"],
    },
    Case {
        name: "singleton-type",
        config: "rules:\n  no-singleton-type: \"off\"\n  types-from-allowlist:\n    severity: error\n    options: { allow: [Alpha] }\n",
        conformant: true,
        codes: &["types-from-allowlist"],
    },
];

#[test]
fn each_case_triggers_exactly_its_target_rules() {
    for case in CASES {
        let files = read_md(&case_dir(case.name));
        let config = if case.config.is_empty() {
            ResolvedConfig::recommended()
        } else {
            ResolvedConfig::from_yaml(case.config)
                .unwrap_or_else(|e| panic!("{}: config: {e}", case.name))
        };
        let bundle = check(files, &config);
        assert_eq!(
            bundle.conformant, case.conformant,
            "case `{}`: conformant",
            case.name
        );
        let got: BTreeSet<&str> = bundle.diagnostics.iter().map(|d| d.code.as_str()).collect();
        let want: BTreeSet<&str> = case.codes.iter().copied().collect();
        assert_eq!(got, want, "case `{}` produced unexpected codes", case.name);
    }
}
