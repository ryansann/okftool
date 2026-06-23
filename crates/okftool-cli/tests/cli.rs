//! End-to-end CLI tests: drive the built binary and assert exit codes + output.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_okftool"))
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/.vendor")
        .join(name)
}

/// Fixtures are gitignored; skip fixture-backed tests when absent.
fn require(name: &str) -> Option<PathBuf> {
    let path = fixture(name);
    path.exists().then_some(path)
}

/// The committed curated corpus (always present, CI-safe).
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cases/all-rules")
}

fn case(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/cases")
        .join(name)
}

fn tmp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("okftool-{}-{}", std::process::id(), name))
}

#[test]
fn build_packages_a_conformant_bundle() {
    let out = tmp("build.tar.gz");
    let _ = std::fs::remove_file(&out);
    let res = bin()
        .arg("build")
        .arg(corpus())
        .arg("-o")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        res.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&res.stderr)
    );
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        bytes.len() > 2 && bytes[0] == 0x1f && bytes[1] == 0x8b,
        "output is not a gzip archive"
    );
    std::fs::remove_file(&out).ok();
}

#[test]
fn build_refuses_nonconformant_bundle() {
    let out = tmp("bad.tar.gz");
    let _ = std::fs::remove_file(&out);
    let res = bin()
        .arg("build")
        .arg(case("missing-type"))
        .arg("-o")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        !res.status.success(),
        "build must refuse a non-conformant bundle"
    );
    assert!(String::from_utf8_lossy(&res.stderr).contains("not OKF-conformant"));
    std::fs::remove_file(&out).ok();
}

/// okftool's own documentation is an OKF bundle; it must stay conformant and
/// lint-clean under the **strict** profile (via `docs/okf/.okftool.yaml`).
fn self_docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/okf")
}

#[test]
fn self_documentation_bundle_is_clean() {
    let validate = bin().arg("validate").arg(self_docs()).output().unwrap();
    assert!(validate.status.success(), "self-docs must be conformant");
    // No --config: okftool picks up docs/okf/.okftool.yaml (extends okf-strict).
    let lint = bin().arg("lint").arg(self_docs()).output().unwrap();
    assert!(
        lint.status.success(),
        "self-docs must be lint-clean under okf-strict:\n{}",
        String::from_utf8_lossy(&lint.stdout)
    );
}

#[test]
fn validate_sample_corpus_is_conformant() {
    let out = bin().arg("validate").arg(corpus()).output().unwrap();
    assert!(out.status.success(), "sample corpus must validate clean");
    assert!(String::from_utf8_lossy(&out.stdout).contains("CONFORMANT"));
}

#[test]
fn lint_sample_corpus_trips_on_error_rule() {
    // The corpus contains a bad timestamp (timestamp-iso8601 = error), so with
    // the default fail-on=error the lint must exit non-zero — even though the
    // bundle is spec-conformant.
    let out = bin().arg("lint").arg(corpus()).output().unwrap();
    assert!(
        !out.status.success(),
        "an error-level lint finding must fail CI"
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("CONFORMANT"));
}

#[test]
fn validate_conformant_fixture_exits_zero() {
    let Some(ga4) = require("ga4") else { return };
    let out = bin().arg("validate").arg(ga4).output().unwrap();
    assert!(out.status.success(), "ga4 should validate clean");
    assert!(String::from_utf8_lossy(&out.stdout).contains("CONFORMANT"));
}

#[test]
fn lint_warns_only_is_conformant_and_exits_zero() {
    let Some(ga4) = require("ga4") else { return };
    let out = bin().arg("lint").arg(ga4).output().unwrap();
    assert!(out.status.success(), "warnings alone must not fail CI");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("CONFORMANT"));
    assert!(stdout.contains("lint/"), "expected lint findings on ga4");
}

#[test]
fn validate_non_conformant_exits_nonzero() {
    let dir = tempdir();
    std::fs::write(dir.join("bad.md"), "---\ntitle: x\n---\nbody\n").unwrap();
    let out = bin().arg("validate").arg(&dir).output().unwrap();
    assert!(!out.status.success(), "missing type must fail validate");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn rules_and_explain_work() {
    let rules = bin().arg("rules").output().unwrap();
    assert!(rules.status.success());
    assert!(String::from_utf8_lossy(&rules.stdout).contains("no-orphan-concepts"));

    let explain = bin().args(["explain", "max-out-degree"]).output().unwrap();
    assert!(explain.status.success());
    assert!(String::from_utf8_lossy(&explain.stdout).contains("hairball"));

    let unknown = bin().args(["explain", "nope"]).output().unwrap();
    assert!(!unknown.status.success());
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("okftool-cli-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
