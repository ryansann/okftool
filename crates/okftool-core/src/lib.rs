//! okftool-core — the OKF parser, spec validator, and lint engine.
//!
//! This crate is the single source of truth shared by every okftool surface:
//! the native CLI, the wasm npm package, and any Rust embedder. It performs no
//! IO — callers supply `(path, content)` pairs — so it compiles unchanged to
//! native and `wasm32`.
//!
//! Two layers, mirroring the OKF spec's MUST/SHOULD split:
//! - [`build_bundle`] parses + enforces §9 conformance (non-disableable errors).
//! - [`lint`] enforces advisory, configurable rules (never affects `conformant`).
//! - [`check`] runs both and returns one [`Bundle`].

mod bundle;
mod config;
mod links;
mod lint;
mod model;
mod parse;

pub use bundle::build_bundle;
pub use config::{parse_severity, ResolvedConfig};
pub use links::{extract_links, is_external, resolve_target, RawLink, ResolvedTarget};
pub use lint::{
    all_rules, canonical_rule_id, lint, rule_descriptors, rule_meta, rule_metas, Category,
    CategoryDescriptor, Finding, Graph, GraphEdge, LintContext, Rule, RuleDescriptor, RuleMeta,
};
pub use model::{
    Bundle, Concept, Diagnostic, IndexEntry, IndexFile, IndexSection, Link, LogDay, LogEntry,
    LogFile, Severity,
};
pub use parse::{
    basename, dir_of, id_from_path, is_reserved, parse_concept, parse_index, parse_log,
    reserved_kind, ParsedIndex, Reserved,
};

/// OKF spec version this build targets.
pub const OKF_VERSION: &str = "0.1";

/// Parse, validate (§9), and lint in one pass. Lint diagnostics are appended to
/// the spec diagnostics; `conformant` reflects spec conformance only.
pub fn check<I, P, C>(files: I, config: &ResolvedConfig) -> Bundle
where
    I: IntoIterator<Item = (P, C)>,
    P: AsRef<str>,
    C: AsRef<str>,
{
    let mut bundle = build_bundle(files);
    let findings = lint(&bundle, config);
    bundle.diagnostics.extend(findings);
    bundle
}
