//! Frontmatter + body parsing for a single OKF document.
//!
//! Deliberately filesystem-agnostic: callers pass `(path, content)` so the same
//! code runs in the CLI (reads files) and in wasm (receives content from JS).

use crate::model::{Concept, Diagnostic};

/// Reserved filenames that are not concepts: `index.md` (table of contents) and
/// `log.md` (change history). Their basename match is case-sensitive per spec.
pub fn is_reserved(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    base == "index.md" || base == "log.md"
}

/// `concepts/thing.md` -> `concepts/thing`.
pub fn id_from_path(path: &str) -> String {
    path.strip_suffix(".md").unwrap_or(path).to_string()
}

/// Split a leading `---`-fenced YAML frontmatter block from the body. Returns
/// `None` when there is no well-formed (opened *and* closed) frontmatter block.
fn split_frontmatter(content: &str) -> Option<(String, String)> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = content.lines();
    match lines.next() {
        Some(first) if first.trim_end() == "---" => {}
        _ => return None,
    }
    let mut frontmatter = String::new();
    let mut closed = false;
    let mut body: Vec<&str> = Vec::new();
    for line in lines {
        if !closed && line.trim_end() == "---" {
            closed = true;
            continue;
        }
        if closed {
            body.push(line);
        } else {
            frontmatter.push_str(line);
            frontmatter.push('\n');
        }
    }
    if !closed {
        return None; // opened but never closed -> not valid frontmatter
    }
    Some((frontmatter, body.join("\n")))
}

/// Convert a parsed YAML value into a JSON object, dropping it to empty if the
/// top level is not a string-keyed mapping.
fn yaml_to_json_object(value: &serde_yaml::Value) -> serde_json::Map<String, serde_json::Value> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    }
}

/// Parse one document into a [`Concept`] plus any spec (§9) diagnostics it raises.
///
/// Reserved files (`index.md`/`log.md`) are parsed leniently here; their
/// structural rules are handled by the validator/lint layer.
pub fn parse_concept(path: &str, content: &str) -> (Concept, Vec<Diagnostic>) {
    let reserved = is_reserved(path);
    let mut diagnostics = Vec::new();

    let (frontmatter, concept_type, title, body) = match split_frontmatter(content) {
        Some((raw, body)) => match serde_yaml::from_str::<serde_yaml::Value>(&raw) {
            Ok(value) => {
                let map = yaml_to_json_object(&value);
                let concept_type = map
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let title = map
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                if !reserved && concept_type.is_none() {
                    diagnostics.push(Diagnostic::spec(
                        path,
                        "missing-type",
                        "Frontmatter is missing a non-empty `type` (the only required OKF field).",
                        "Add a `type:` field, e.g. `type: BigQuery Table`.",
                    ));
                }
                (map, concept_type, title, body)
            }
            Err(err) => {
                diagnostics.push(Diagnostic::spec(
                    path,
                    "frontmatter-parse",
                    format!("Frontmatter YAML failed to parse: {err}"),
                    "Fix the YAML syntax — check indentation, colons, and quoting.",
                ));
                (serde_json::Map::new(), None, None, body)
            }
        },
        None => {
            if !reserved {
                diagnostics.push(Diagnostic::spec(
                    path,
                    "missing-frontmatter",
                    "File has no YAML frontmatter block delimited by `---`.",
                    "Add a `---`-delimited YAML frontmatter block at the top, with at least `type:`.",
                ));
            }
            (serde_json::Map::new(), None, None, content.to_string())
        }
    };

    let concept = Concept {
        id: id_from_path(path),
        path: path.to_string(),
        concept_type,
        title,
        frontmatter,
        body,
    };
    (concept, diagnostics)
}
