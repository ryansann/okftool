//! Data model shared across the validator, the lint engine, and every binding.
//!
//! Everything here is `serde`-serializable so the wasm binding can hand it to
//! JavaScript and the CLI can emit JSON/SARIF without a second model.

use serde::{Deserialize, Serialize};

/// Diagnostic severity. `Error` is reserved for spec (§9) conformance failures;
/// lint rules emit `Info`/`Warn`/`Error` per their configured level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Info,
    Warn,
    Error,
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
    /// Whether this diagnostic comes from the spec validator (`true`) or a lint
    /// rule (`false`). Validator diagnostics are never disableable.
    pub spec: bool,
    /// Optional human-actionable suggestion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

/// One parsed OKF concept document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Concept id = bundle-relative path minus `.md`.
    pub id: String,
    /// Bundle-relative path including `.md`.
    pub path: String,
    /// The required `type` field, if present and non-empty.
    #[serde(rename = "type")]
    pub concept_type: Option<String>,
    /// Convenience accessor for the `title` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Full frontmatter as a JSON object (keys preserved, per spec).
    pub frontmatter: serde_json::Map<String, serde_json::Value>,
    /// Markdown body following the frontmatter block.
    pub body: String,
}

/// The result of parsing + validating a directory of files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub concepts: Vec<Concept>,
    pub diagnostics: Vec<Diagnostic>,
    /// True iff no spec (§9) error is present. Lint findings never affect this.
    pub conformant: bool,
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
