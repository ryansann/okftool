//! `.okftool.yaml` configuration: presets (`extends`), per-rule severity/options,
//! glob `overrides`, and the CI `fail-on` gate. Resolves the ESLint-style cascade
//! preset → root config → path override → (inline disable, applied by the engine).

use std::collections::HashMap;

use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use serde_json::Value;

use crate::lint::canonical_rule_id;
use crate::model::{Concept, Severity};

/// Parse a severity word; `off`/`info`/`warn`/`error` (case-insensitive).
pub fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_ascii_lowercase().as_str() {
        "off" => Some(Severity::Off),
        "info" => Some(Severity::Info),
        "warn" | "warning" => Some(Severity::Warn),
        "error" => Some(Severity::Error),
        _ => None,
    }
}

// ---- raw (deserialized) shapes -------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RuleSetting {
    // NB: YAML 1.1 parses bare `off`/`on` as booleans, so accept that form too.
    Bool(bool),
    Severity(String),
    Detailed {
        severity: Option<String>,
        #[serde(default)]
        options: Value,
    },
}

impl RuleSetting {
    /// `(explicit severity, options)`.
    fn split(&self) -> (Option<Severity>, Value) {
        match self {
            RuleSetting::Bool(false) => (Some(Severity::Off), Value::Null),
            RuleSetting::Bool(true) => (None, Value::Null),
            RuleSetting::Severity(s) => (parse_severity(s), Value::Null),
            RuleSetting::Detailed { severity, options } => (
                severity.as_deref().and_then(parse_severity),
                options.clone(),
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Extends {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    extends: Option<Extends>,
    #[allow(dead_code)]
    okf_version: Option<String>,
    rules: HashMap<String, RuleSetting>,
    graph: Option<RawGraph>,
    overrides: Vec<RawOverride>,
    ci: Option<RawCi>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawGraph {
    neighborhoods: HashMap<String, RawNeighborhood>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawNeighborhood {
    paths: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawOverride {
    files: String,
    rules: HashMap<String, RuleSetting>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawCi {
    #[serde(rename = "fail-on")]
    fail_on: Option<String>,
    report: Option<String>,
}

// ---- resolved config -----------------------------------------------------------

struct RuleConfig {
    severity: Option<Severity>,
    options: Value,
}

struct OverrideConfig {
    matcher: GlobMatcher,
    rules: HashMap<String, Option<Severity>>,
}

/// Fully resolved configuration ready for the lint engine.
pub struct ResolvedConfig {
    rules: HashMap<String, RuleConfig>,
    neighborhoods: HashMap<String, String>,
    neighborhood_globs: Vec<NeighborhoodGlob>,
    overrides: Vec<OverrideConfig>,
    /// Severity at/above which `lint` should fail CI (default `error`).
    pub fail_on: Severity,
    /// Requested report format from `ci.report`, if any.
    pub report: Option<String>,
}

struct NeighborhoodGlob {
    matcher: GlobMatcher,
    name: String,
}

fn preset_yaml(name: &str) -> Option<&'static str> {
    match name {
        "okf-recommended" => Some(include_str!("../presets/okf-recommended.yaml")),
        "okf-strict" => Some(include_str!("../presets/okf-strict.yaml")),
        "okf-minimal" => Some(include_str!("../presets/okf-minimal.yaml")),
        _ => None,
    }
}

impl ResolvedConfig {
    /// The default profile (every rule at its built-in severity).
    pub fn recommended() -> Self {
        Self::resolve(RawConfig::default())
    }

    /// Parse `.okftool.yaml` text into a resolved config.
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let raw: RawConfig = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
        Ok(Self::resolve(raw))
    }

    fn resolve(raw: RawConfig) -> Self {
        let mut rules: HashMap<String, RuleConfig> = HashMap::new();

        // Apply presets (extends) as the base layer.
        let extend_names: Vec<String> = match raw.extends {
            Some(Extends::One(s)) => vec![s],
            Some(Extends::Many(v)) => v,
            None => Vec::new(),
        };
        for name in extend_names {
            if let Some(yaml) = preset_yaml(&name) {
                if let Ok(preset) = serde_yaml::from_str::<RawConfig>(yaml) {
                    for (id, setting) in preset.rules {
                        let (severity, options) = setting.split();
                        let id = normalize_rule_id(&id);
                        rules.insert(id, RuleConfig { severity, options });
                    }
                }
            }
        }

        // Root config rules override the preset layer.
        for (id, setting) in raw.rules {
            let (severity, options) = setting.split();
            let id = normalize_rule_id(&id);
            rules.insert(id, RuleConfig { severity, options });
        }

        let mut neighborhoods = HashMap::new();
        let mut neighborhood_globs = Vec::new();
        if let Some(graph) = raw.graph {
            for (name, neighborhood) in graph.neighborhoods {
                for path in neighborhood.paths {
                    if is_glob_path(&path) {
                        if let Ok(matcher) = Glob::new(&path).map(|glob| glob.compile_matcher()) {
                            neighborhood_globs.push(NeighborhoodGlob {
                                matcher,
                                name: name.clone(),
                            });
                        }
                    } else {
                        neighborhoods.insert(normalize_concept_path(&path), name.clone());
                    }
                }
            }
        }

        let overrides = raw
            .overrides
            .iter()
            .filter_map(|o| {
                let matcher = Glob::new(&o.files).ok()?.compile_matcher();
                let rules = o
                    .rules
                    .iter()
                    .map(|(id, s)| (normalize_rule_id(id), s.split().0))
                    .collect();
                Some(OverrideConfig { matcher, rules })
            })
            .collect();

        let fail_on = raw
            .ci
            .as_ref()
            .and_then(|c| c.fail_on.as_deref())
            .and_then(parse_severity)
            .unwrap_or(Severity::Error);
        let report = raw.ci.and_then(|c| c.report);

        ResolvedConfig {
            rules,
            neighborhoods,
            neighborhood_globs,
            overrides,
            fail_on,
            report,
        }
    }

    /// Infer a concept's graph-structure neighborhood.
    ///
    /// Precedence: explicit config path mapping, concept frontmatter
    /// `neighborhood`, directory prefix, then a per-root-concept neighborhood.
    pub fn neighborhood_for(&self, concept: &Concept) -> String {
        if let Some(name) = self.neighborhoods.get(&concept.id) {
            return name.clone();
        }
        for configured in &self.neighborhood_globs {
            if configured.matcher.is_match(&concept.path)
                || configured.matcher.is_match(&concept.id)
            {
                return configured.name.clone();
            }
        }
        if let Some(name) = concept
            .frontmatter
            .get("neighborhood")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return name.to_string();
        }
        match concept.id.rfind('/') {
            Some(i) => concept.id[..i].to_string(),
            None => format!("(root)/{}", concept.id),
        }
    }

    /// Options object configured for a rule (`Null` if none).
    pub fn options(&self, id: &str) -> Value {
        self.rules
            .get(id)
            .map(|r| r.options.clone())
            .unwrap_or(Value::Null)
    }

    /// Configured severity for a rule before path/inline scoping (preset/root or
    /// the rule's built-in default).
    pub fn base_severity(&self, id: &str, default: Severity) -> Severity {
        self.rules
            .get(id)
            .and_then(|r| r.severity)
            .unwrap_or(default)
    }

    /// Effective severity for a rule on a specific file, applying path overrides
    /// (most-specific/last wins) and an inline disable flag.
    pub fn effective_severity(
        &self,
        id: &str,
        file: &str,
        default: Severity,
        inline_disabled: bool,
    ) -> Severity {
        if inline_disabled {
            return Severity::Off;
        }
        let mut severity = self.base_severity(id, default);
        for ov in &self.overrides {
            if ov.matcher.is_match(file) {
                if let Some(s) = ov
                    .rules
                    .get(id)
                    .copied()
                    .flatten()
                    .or_else(|| ov.rules.get("*").copied().flatten())
                {
                    severity = s;
                }
            }
        }
        severity
    }
}

fn normalize_rule_id(id: &str) -> String {
    if id == "*" {
        return id.to_string();
    }
    canonical_rule_id(id).unwrap_or(id).to_string()
}

fn normalize_concept_path(path: &str) -> String {
    path.trim_start_matches('/')
        .strip_suffix(".md")
        .unwrap_or_else(|| path.trim_start_matches('/'))
        .to_string()
}

fn is_glob_path(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[') || path.contains('{')
}
