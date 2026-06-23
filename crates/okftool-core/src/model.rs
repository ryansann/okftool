//! Data model shared across the validator, the lint engine, and every binding.
//!
//! Everything here is `serde`-serializable (camelCase for JS ergonomics) so the
//! wasm binding can hand it to JavaScript and the CLI can emit JSON/SARIF without
//! a second model.

use serde::{Deserialize, Serialize};

/// Diagnostic severity. Spec (§9) conformance failures are always `Error`; lint
/// rules emit `Info`/`Warn`/`Error` per their configured level (`Off` = silent).
///
/// Declaration order is ascending, so `Off < Info < Warn < Error` via `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Info,
    Warn,
    Error,
}

impl Severity {
    /// SARIF level (`error`/`warning`/`note`); `Off` maps to `none`.
    pub fn sarif_level(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warn => "warning",
            Severity::Info => "note",
            Severity::Off => "none",
        }
    }

    /// Lower-case label for plain-text output.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
            Severity::Off => "off",
        }
    }
}

/// A single finding against one file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Bundle-relative path of the offending file (e.g. `concepts/thing.md`).
    pub file: String,
    /// Stable machine code (e.g. `missing-type`, `no-orphan-concepts`).
    pub code: String,
    pub severity: Severity,
    pub message: String,
    /// `true` for spec-validator diagnostics (never disableable), `false` for
    /// lint rules.
    pub spec: bool,
    /// Optional human-actionable suggestion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

impl Diagnostic {
    pub(crate) fn spec(file: &str, code: &str, message: impl Into<String>, fix: &str) -> Self {
        Diagnostic {
            file: file.to_string(),
            code: code.to_string(),
            severity: Severity::Error,
            message: message.into(),
            spec: true,
            fix: Some(fix.to_string()),
        }
    }
}

/// A markdown link from a concept body, resolved against the bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// Raw markdown href.
    pub href: String,
    pub text: String,
    /// Resolved in-bundle concept id (`.md` stripped), if the target exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    /// Resolved external URL, if external.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external: Option<String>,
    /// An in-bundle link whose target concept does not exist.
    pub broken: bool,
}

/// One parsed OKF concept document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Concept {
    /// Concept id = bundle-relative path minus `.md`.
    pub id: String,
    /// Bundle-relative path including `.md`.
    pub path: String,
    /// The required `type` field, if present and non-empty.
    #[serde(rename = "type")]
    pub concept_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Full frontmatter as a JSON object (all keys preserved, per spec).
    pub frontmatter: serde_json::Map<String, serde_json::Value>,
    /// Markdown body following the frontmatter block.
    pub body: String,
    pub outgoing: Vec<Link>,
}

/// One entry in an `index.md` table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub title: String,
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSection {
    pub heading: String,
    pub entries: Vec<IndexEntry>,
}

/// A reserved `index.md` (directory table of contents).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexFile {
    /// Directory the index describes (`""` = bundle root).
    pub dir: String,
    pub path: String,
    pub sections: Vec<IndexSection>,
    /// Whether the file carried a frontmatter block (only valid in the root).
    pub has_frontmatter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verb: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDay {
    pub date: String,
    pub entries: Vec<LogEntry>,
}

/// A reserved `log.md` (change history).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFile {
    pub path: String,
    pub dir: String,
    pub days: Vec<LogDay>,
}

/// The result of parsing + validating a directory of files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub concepts: Vec<Concept>,
    pub indexes: Vec<IndexFile>,
    pub logs: Vec<LogFile>,
    /// Distinct, sorted, non-empty concept types.
    pub types: Vec<String>,
    /// Declared in the bundle-root `index.md` only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okf_version: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
    /// True iff no spec (§9) error is present. Lint findings never affect this.
    pub conformant: bool,
}
