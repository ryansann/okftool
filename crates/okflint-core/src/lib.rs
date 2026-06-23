//! okflint-core — the OKF parser, spec validator, and (forthcoming) lint engine.
//!
//! This crate is the single source of truth shared by every okflint surface:
//! the native CLI, the wasm npm package, and any Rust embedder. It performs no
//! IO — callers supply `(path, content)` pairs — so it compiles unchanged to
//! native and `wasm32`.
//!
//! Two layers, mirroring the OKF spec's MUST/SHOULD split:
//! - [`validate_files`] enforces §9 conformance (non-disableable errors).
//! - the lint engine (Phase 2) enforces advisory, configurable rules.

mod model;
mod parse;
mod validate;

pub use model::{Bundle, Concept, Diagnostic, Severity};
pub use parse::{id_from_path, is_reserved, parse_concept};
pub use validate::validate_files;

/// OKF spec version this build targets.
pub const OKF_VERSION: &str = "0.1";
