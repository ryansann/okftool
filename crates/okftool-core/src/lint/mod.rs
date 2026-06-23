//! The lint engine: advisory, configurable rules over a parsed [`Bundle`].
//!
//! Each [`Rule`] declares [`RuleMeta`] (the data behind `--explain`, presets, and
//! config validation) and a `check`. The engine resolves every finding's
//! severity through the config cascade (preset → root → path override → inline
//! disable) and drops anything resolving to `Off`. Lint never affects `conformant`.

mod rules;

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::config::ResolvedConfig;
use crate::model::{Bundle, Diagnostic, Severity};

/// Rule grouping, for `--explain` and docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Frontmatter,
    TypeVocabulary,
    Linking,
    Topology,
    Body,
    IndexLog,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Frontmatter => "frontmatter",
            Category::TypeVocabulary => "type-vocabulary",
            Category::Linking => "linking",
            Category::Topology => "topology",
            Category::Body => "body",
            Category::IndexLog => "index-log",
        }
    }
}

/// Static description of a rule — the manifest entry.
pub struct RuleMeta {
    pub id: &'static str,
    pub category: Category,
    pub summary: &'static str,
    pub rationale: &'static str,
    pub default_severity: Severity,
    pub fixable: bool,
}

/// A rule finding before severity resolution.
pub struct Finding {
    pub file: String,
    pub message: String,
    pub fix: Option<String>,
}

impl Finding {
    pub fn new(file: impl Into<String>, message: impl Into<String>) -> Self {
        Finding {
            file: file.into(),
            message: message.into(),
            fix: None,
        }
    }

    pub fn with_fix(
        file: impl Into<String>,
        message: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Finding {
            file: file.into(),
            message: message.into(),
            fix: Some(fix.into()),
        }
    }
}

/// Precomputed link-graph degrees (self-links and external/broken links excluded).
pub struct Graph {
    pub out_degree: HashMap<String, usize>,
    pub in_degree: HashMap<String, usize>,
}

impl Graph {
    fn build(bundle: &Bundle) -> Self {
        let ids: HashSet<&str> = bundle.concepts.iter().map(|c| c.id.as_str()).collect();
        let mut out_degree: HashMap<String, usize> =
            bundle.concepts.iter().map(|c| (c.id.clone(), 0)).collect();
        let mut in_degree: HashMap<String, usize> = out_degree.clone();

        for concept in &bundle.concepts {
            let mut seen = HashSet::new();
            for link in &concept.outgoing {
                if let Some(target) = &link.target_id {
                    if target != &concept.id
                        && ids.contains(target.as_str())
                        && seen.insert(target.as_str())
                    {
                        *out_degree.get_mut(&concept.id).unwrap() += 1;
                        *in_degree.get_mut(target).unwrap() += 1;
                    }
                }
            }
        }
        Graph {
            out_degree,
            in_degree,
        }
    }
}

/// What a rule sees while checking.
pub struct LintContext<'a> {
    pub bundle: &'a Bundle,
    pub graph: &'a Graph,
    /// This rule's configured options (`Null` if none).
    pub options: &'a Value,
}

/// A lint rule.
pub trait Rule: Sync {
    fn meta(&self) -> RuleMeta;
    fn check(&self, ctx: &LintContext) -> Vec<Finding>;
}

/// Every registered rule, in stable display order.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    rules::registry()
}

/// The manifest: every rule's metadata.
pub fn rule_metas() -> Vec<RuleMeta> {
    all_rules().iter().map(|r| r.meta()).collect()
}

/// Inline `okf-lint-disable` rule ids per concept path.
fn inline_disables(bundle: &Bundle) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    for concept in &bundle.concepts {
        let set: HashSet<String> = match concept.frontmatter.get("okf-lint-disable") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            Some(Value::String(s)) => std::iter::once(s.clone()).collect(),
            _ => continue,
        };
        if !set.is_empty() {
            map.insert(concept.path.clone(), set);
        }
    }
    map
}

/// Run all lint rules against a bundle, returning resolved diagnostics
/// (`spec == false`). Findings resolving to `Off` are dropped.
pub fn lint(bundle: &Bundle, config: &ResolvedConfig) -> Vec<Diagnostic> {
    let graph = Graph::build(bundle);
    let disabled = inline_disables(bundle);
    let mut out = Vec::new();

    for rule in all_rules() {
        let meta = rule.meta();
        let options = config.options(meta.id);
        let ctx = LintContext {
            bundle,
            graph: &graph,
            options: &options,
        };
        for finding in rule.check(&ctx) {
            let inline = disabled
                .get(&finding.file)
                .is_some_and(|s| s.contains(meta.id));
            let severity =
                config.effective_severity(meta.id, &finding.file, meta.default_severity, inline);
            if severity == Severity::Off {
                continue;
            }
            out.push(Diagnostic {
                file: finding.file,
                code: meta.id.to_string(),
                severity,
                message: finding.message,
                spec: false,
                fix: finding.fix,
            });
        }
    }
    out
}
