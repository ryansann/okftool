//! WebAssembly bindings — the `okflint` npm package consumed by the desktop app,
//! a Node/edge API, or the browser. JS passes file contents in (wasm has no
//! filesystem); okflint-core does the rest.

use okflint_core::validate_files;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

/// One input file. JS shape: `{ path: string, content: string }`.
#[derive(Deserialize)]
struct FileInput {
    path: String,
    content: String,
}

/// Initialize panic forwarding to `console.error`. Safe to call more than once.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Validate a bundle against the OKF spec (§9).
///
/// `files` is `Array<{ path, content }>`; returns the serialized [`Bundle`].
///
/// [`Bundle`]: okflint_core::Bundle
#[wasm_bindgen]
pub fn validate(files: JsValue) -> Result<JsValue, JsValue> {
    let inputs: Vec<FileInput> = serde_wasm_bindgen::from_value(files)
        .map_err(|e| JsValue::from_str(&format!("invalid input: {e}")))?;
    let bundle = validate_files(inputs.into_iter().map(|f| (f.path, f.content)));
    serde_wasm_bindgen::to_value(&bundle).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// The OKF spec version this build targets.
#[wasm_bindgen]
pub fn okf_version() -> String {
    okflint_core::OKF_VERSION.to_string()
}
