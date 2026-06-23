//! Frontmatter + body parsing for OKF documents. Filesystem-agnostic: callers
//! pass `(path, content)` so the same code runs in the CLI and in wasm.
//!
//! Behavior mirrors okfview's `parse.ts` so okflint's verdict matches the app's.

use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Map, Value};

use crate::links::extract_links;
use crate::model::{
    Concept, Diagnostic, IndexEntry, IndexFile, IndexSection, Link, LogDay, LogEntry, LogFile,
};

/// Lower-cased final path segment.
pub fn basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_lowercase()
}

/// Directory portion of a path (`""` for a top-level file).
pub fn dir_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(i) => &path[..i],
        None => "",
    }
}

/// `concepts/Thing.MD` -> `concepts/Thing` (case-insensitive extension).
pub fn id_from_path(path: &str) -> String {
    if path.len() >= 3 && path[path.len() - 3..].eq_ignore_ascii_case(".md") {
        path[..path.len() - 3].to_string()
    } else {
        path.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reserved {
    Index,
    Log,
}

/// Classify a reserved filename (`index.md` / `log.md`), else `None`.
pub fn reserved_kind(path: &str) -> Option<Reserved> {
    match basename(path).as_str() {
        "index.md" => Some(Reserved::Index),
        "log.md" => Some(Reserved::Log),
        _ => None,
    }
}

pub fn is_reserved(path: &str) -> bool {
    reserved_kind(path).is_some()
}

/// True iff `content` opens with a `---` frontmatter fence (optional BOM), per
/// okfview's `/^\u{feff}?---\r?\n/` detection.
fn has_frontmatter(content: &str) -> bool {
    let c = content.strip_prefix('\u{feff}').unwrap_or(content);
    let first = c.lines().next().unwrap_or("");
    first == "---"
}

/// Split a leading `---`-fenced YAML block from the body. Returns `(yaml, body)`
/// or `None` when there is no opened-and-closed frontmatter.
fn split_frontmatter(content: &str) -> Option<(String, String)> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return None;
    }
    let mut yaml = String::new();
    let mut body: Vec<&str> = Vec::new();
    let mut closed = false;
    for line in lines {
        if !closed && line == "---" {
            closed = true;
            continue;
        }
        if closed {
            body.push(line);
        } else {
            yaml.push_str(line);
            yaml.push('\n');
        }
    }
    if !closed {
        return None;
    }
    Some((yaml, body.join("\n")))
}

fn yaml_to_object(yaml: &str) -> Result<Map<String, Value>, String> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    match serde_json::to_value(&value) {
        Ok(Value::Object(map)) => Ok(map),
        // Empty frontmatter parses to null; treat as an empty object.
        Ok(Value::Null) => Ok(Map::new()),
        Ok(_) => Ok(Map::new()),
        Err(e) => Err(e.to_string()),
    }
}

/// Coerce a frontmatter scalar to a string (mirrors `asString`).
fn as_string(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

/// Coerce a frontmatter value to a tag list (mirrors `asTags`).
fn as_tags(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .map(|x| match x {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect(),
        Some(Value::String(s)) => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse a non-reserved concept document. Always returns a [`Concept`]
/// (permissive) plus any spec (§9) diagnostics.
pub fn parse_concept(path: &str, raw: &str) -> (Concept, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut frontmatter = Map::new();
    let mut body = raw.to_string();

    if !has_frontmatter(raw) {
        diagnostics.push(Diagnostic::spec(
            path,
            "missing-frontmatter",
            "Concept document has no YAML frontmatter block (spec §9.1).",
            "Add a `---`-delimited YAML frontmatter block at the top, with at least `type:`.",
        ));
    } else if let Some((yaml, parsed_body)) = split_frontmatter(raw) {
        body = parsed_body;
        match yaml_to_object(&yaml) {
            Ok(map) => frontmatter = map,
            Err(err) => diagnostics.push(Diagnostic::spec(
                path,
                "frontmatter-parse",
                format!("Unparseable YAML frontmatter: {err}"),
                "Fix the YAML syntax — check indentation, colons, and quoting.",
            )),
        }
    } else {
        // Opened a fence but never closed it — gray-matter would treat the whole
        // file as body; match that and flag the missing block.
        diagnostics.push(Diagnostic::spec(
            path,
            "missing-frontmatter",
            "Concept document has no closed YAML frontmatter block (spec §9.1).",
            "Close the frontmatter block with a `---` line.",
        ));
    }

    let concept_type = as_string(frontmatter.get("type"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if concept_type.is_none() {
        diagnostics.push(Diagnostic::spec(
            path,
            "missing-type",
            "Frontmatter is missing a non-empty `type` field (spec §9.2).",
            "Add a `type:` field, e.g. `type: BigQuery Table`.",
        ));
    }

    let outgoing = extract_links(&body)
        .into_iter()
        .map(|l| Link {
            href: l.href,
            text: l.text,
            target_id: None,
            external: None,
            broken: false,
        })
        .collect();

    let concept = Concept {
        id: id_from_path(path),
        path: path.to_string(),
        title: as_string(frontmatter.get("title")),
        description: as_string(frontmatter.get("description")),
        resource: as_string(frontmatter.get("resource")),
        tags: as_tags(frontmatter.get("tags")),
        timestamp: as_string(frontmatter.get("timestamp")),
        concept_type,
        frontmatter,
        body,
        outgoing,
    };
    (concept, diagnostics)
}

/// Result of parsing a reserved `index.md`.
pub struct ParsedIndex {
    pub index: IndexFile,
    pub okf_version: Option<String>,
}

fn heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#{1,6}\s+(.*?)\s*$").unwrap())
}

fn list_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\s*[*+-]\s+\[([^\]]*)\]\(([^)]+)\)\s*(?:[-–—:]\s*(.*))?$").unwrap()
    })
}

/// Parse a reserved `index.md`. Frontmatter is only valid in the bundle root.
pub fn parse_index(path: &str, raw: &str) -> ParsedIndex {
    let mut okf_version = None;
    let mut body = raw.to_string();
    let had_frontmatter = has_frontmatter(raw);

    if had_frontmatter {
        if let Some((yaml, parsed_body)) = split_frontmatter(raw) {
            body = parsed_body;
            if let Ok(map) = yaml_to_object(&yaml) {
                if let Some(v) = map.get("okf_version") {
                    okf_version = as_string(Some(v));
                }
            }
        }
    }

    let mut sections: Vec<IndexSection> = Vec::new();
    for line in body.lines() {
        if let Some(h) = heading_re().captures(line) {
            sections.push(IndexSection {
                heading: h[1].to_string(),
                entries: Vec::new(),
            });
            continue;
        }
        if let Some(li) = list_link_re().captures(line) {
            if sections.is_empty() {
                sections.push(IndexSection {
                    heading: String::new(),
                    entries: Vec::new(),
                });
            }
            let entry = IndexEntry {
                title: li[1].to_string(),
                href: li[2].to_string(),
                description: li
                    .get(3)
                    .map(|m| m.as_str().trim().to_string())
                    .filter(|s| !s.is_empty()),
                target_id: None,
            };
            sections.last_mut().unwrap().entries.push(entry);
        }
    }

    ParsedIndex {
        index: IndexFile {
            dir: dir_of(path).to_string(),
            path: path.to_string(),
            sections,
            has_frontmatter: had_frontmatter,
        },
        okf_version,
    }
}

fn log_date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^#{1,6}\s+(\d{4}-\d{2}-\d{2})\s*$").unwrap())
}

fn log_item_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s*[*+-]\s+(.*)$").unwrap())
}

fn log_verb_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\*\*([^*]+)\*\*:?\s*(.*)$").unwrap())
}

/// Parse a reserved `log.md` into dated entries.
pub fn parse_log(path: &str, raw: &str) -> LogFile {
    let mut days: Vec<LogDay> = Vec::new();
    for line in raw.lines() {
        if let Some(d) = log_date_re().captures(line.trim()) {
            days.push(LogDay {
                date: d[1].to_string(),
                entries: Vec::new(),
            });
            continue;
        }
        if let Some(item) = log_item_re().captures(line) {
            if let Some(day) = days.last_mut() {
                let text = &item[1];
                let entry = if let Some(v) = log_verb_re().captures(text) {
                    LogEntry {
                        verb: Some(v[1].trim().to_string()),
                        text: v[2].trim().to_string(),
                    }
                } else {
                    LogEntry {
                        verb: None,
                        text: text.trim().to_string(),
                    }
                };
                day.entries.push(entry);
            }
        }
    }
    LogFile {
        path: path.to_string(),
        dir: dir_of(path).to_string(),
        days,
    }
}
