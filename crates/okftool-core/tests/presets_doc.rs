//! Guard: the preset configs documented in the OKF bundle's `profiles.md` must
//! match the preset YAML compiled into okftool, so the docs can't
//! drift from the source of truth.

use std::fs;
use std::path::Path;

#[test]
fn profiles_doc_matches_presets() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("../..");
    let doc =
        fs::read_to_string(root.join("docs/okf/reference/profiles.md")).expect("read profiles.md");
    for name in ["okf-recommended", "okf-strict", "okf-minimal"] {
        let preset = fs::read_to_string(manifest.join("presets").join(format!("{name}.yaml")))
            .unwrap_or_else(|_| panic!("read presets/{name}.yaml"));
        assert!(
            doc.contains(preset.trim_end()),
            "docs/okf/reference/profiles.md is out of sync with presets/{name}.yaml"
        );
    }
}
