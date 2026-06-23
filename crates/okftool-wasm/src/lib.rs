//! WebAssembly bindings — the `okftool` npm package consumed by the desktop app,
//! a Node/edge API, or the browser. JS passes file contents in (wasm has no
//! filesystem); okftool-core does the rest.

use okftool_core::{build_bundle, check, rule_metas, ResolvedConfig};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

/// One input file. JS shape: `{ path: string, content: string }`.
#[derive(Deserialize)]
struct FileInput {
    path: String,
    content: String,
}

fn parse_files(files: JsValue) -> Result<Vec<(String, String)>, JsValue> {
    let inputs: Vec<FileInput> = serde_wasm_bindgen::from_value(files)
        .map_err(|e| JsValue::from_str(&format!("invalid input: {e}")))?;
    Ok(inputs.into_iter().map(|f| (f.path, f.content)).collect())
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Initialize panic forwarding to `console.error`. Safe to call more than once.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Validate a bundle against the OKF spec (§9). `files` is
/// `Array<{ path, content }>`; returns the serialized `Bundle`.
#[wasm_bindgen]
pub fn validate(files: JsValue) -> Result<JsValue, JsValue> {
    let bundle = build_bundle(parse_files(files)?);
    to_js(&bundle)
}

/// Validate *and* lint a bundle. `config_yaml` is the contents of a
/// `.okftool.yaml` (empty string → the okf-recommended profile). Returns the
/// serialized `Bundle` with spec + lint diagnostics merged.
#[wasm_bindgen]
pub fn lint(files: JsValue, config_yaml: Option<String>) -> Result<JsValue, JsValue> {
    let config = match config_yaml
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(yaml) => ResolvedConfig::from_yaml(yaml).map_err(|e| JsValue::from_str(&e))?,
        None => ResolvedConfig::recommended(),
    };
    let bundle = check(parse_files(files)?, &config);
    to_js(&bundle)
}

/// The rule manifest: `[{ id, category, summary, rationale, defaultSeverity, fixable }]`.
/// Powers an in-app "explain" / rule browser.
#[wasm_bindgen]
pub fn rules() -> Result<JsValue, JsValue> {
    let metas: Vec<_> = rule_metas()
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "category": m.category.as_str(),
                "summary": m.summary,
                "rationale": m.rationale,
                "defaultSeverity": m.default_severity.label(),
                "fixable": m.fixable,
            })
        })
        .collect();
    to_js(&metas)
}

/// The OKF spec version this build targets.
#[wasm_bindgen]
pub fn okf_version() -> String {
    okftool_core::OKF_VERSION.to_string()
}
