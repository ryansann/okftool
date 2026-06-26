//! The lint engine: advisory, configurable rules over a parsed [`Bundle`].
//!
//! Each [`Rule`] declares [`RuleMeta`] (the data behind `--explain`, presets, and
//! config validation) and a `check`. The engine resolves every finding's
//! severity through the config cascade (preset → root → path override → inline
//! disable) and drops anything resolving to `Off`. Lint never affects `conformant`.

mod rules;

use std::collections::{HashMap, HashSet};

use serde::Serialize;
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
    GraphStructure,
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
            Category::GraphStructure => "graph-structure",
            Category::Body => "body",
            Category::IndexLog => "index-log",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Category::Frontmatter => "Frontmatter",
            Category::TypeVocabulary => "Type vocabulary",
            Category::Linking => "Linking",
            Category::Topology => "Topology",
            Category::GraphStructure => "Graph structure",
            Category::Body => "Body",
            Category::IndexLog => "Index & log",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Category::Frontmatter => "Rules about concept metadata quality.",
            Category::TypeVocabulary => "Rules about keeping the concept type vocabulary coherent.",
            Category::Linking => "Rules about individual markdown links and link targets.",
            Category::Topology => "Rules about basic graph health and reachability.",
            Category::GraphStructure => {
                "Rules about coherent local graph shape and cross-neighborhood structure."
            }
            Category::Body => "Rules about concept body structure and retrieval quality.",
            Category::IndexLog => "Rules about reserved index.md and log.md files.",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
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

impl RuleMeta {
    pub fn slug(&self) -> &'static str {
        self.id.rsplit_once('/').map_or(self.id, |(_, slug)| slug)
    }

    pub fn aliases(&self) -> Vec<&'static str> {
        vec![self.slug()]
    }

    pub fn name(&self) -> String {
        self.slug()
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn help(&self) -> &'static str {
        self.rationale
    }

    pub fn category_descriptor(&self) -> CategoryDescriptor {
        CategoryDescriptor {
            id: self.category.as_str(),
            name: self.category.name(),
            description: self.category.description(),
        }
    }

    pub fn docs_path(&self) -> String {
        format!("docs/okf/reference/rules/{}.md", self.slug())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDescriptor {
    pub id: &'static str,
    pub slug: &'static str,
    pub name: String,
    pub aliases: Vec<&'static str>,
    pub category: CategoryDescriptor,
    pub summary: &'static str,
    pub rationale: &'static str,
    pub help: &'static str,
    pub default_severity: &'static str,
    pub fixable: bool,
    pub docs_path: String,
}

impl From<RuleMeta> for RuleDescriptor {
    fn from(meta: RuleMeta) -> Self {
        RuleDescriptor {
            id: meta.id,
            slug: meta.slug(),
            name: meta.name(),
            aliases: meta.aliases(),
            category: meta.category_descriptor(),
            summary: meta.summary,
            rationale: meta.rationale,
            help: meta.help(),
            default_severity: meta.default_severity.label(),
            fixable: meta.fixable,
            docs_path: meta.docs_path(),
        }
    }
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

/// A deduped internal concept edge.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub source_neighborhood: String,
    pub target_neighborhood: String,
}

impl GraphEdge {
    pub fn cohesive(&self) -> bool {
        self.source_neighborhood == self.target_neighborhood
    }

    pub fn bridging(&self) -> bool {
        !self.cohesive()
    }
}

/// Precomputed link-graph facts (self-links and external/broken links excluded).
pub struct Graph {
    pub out_degree: HashMap<String, usize>,
    pub in_degree: HashMap<String, usize>,
    pub local_out_degree: HashMap<String, usize>,
    pub local_in_degree: HashMap<String, usize>,
    pub neighborhoods: HashMap<String, String>,
    pub edges: Vec<GraphEdge>,
    pub outgoing_edges: HashMap<String, Vec<GraphEdge>>,
    pub neighborhood_members: HashMap<String, Vec<String>>,
}

impl Graph {
    fn build(bundle: &Bundle, config: &ResolvedConfig) -> Self {
        let ids: HashSet<&str> = bundle.concepts.iter().map(|c| c.id.as_str()).collect();
        let mut out_degree: HashMap<String, usize> =
            bundle.concepts.iter().map(|c| (c.id.clone(), 0)).collect();
        let mut in_degree: HashMap<String, usize> = out_degree.clone();
        let mut local_out_degree: HashMap<String, usize> = out_degree.clone();
        let mut local_in_degree: HashMap<String, usize> = out_degree.clone();
        let mut neighborhoods: HashMap<String, String> = HashMap::new();
        let mut neighborhood_members: HashMap<String, Vec<String>> = HashMap::new();
        for concept in &bundle.concepts {
            let neighborhood = config.neighborhood_for(concept);
            neighborhoods.insert(concept.id.clone(), neighborhood.clone());
            neighborhood_members
                .entry(neighborhood)
                .or_default()
                .push(concept.id.clone());
        }
        let mut edges = Vec::new();
        let mut outgoing_edges: HashMap<String, Vec<GraphEdge>> = bundle
            .concepts
            .iter()
            .map(|c| (c.id.clone(), Vec::new()))
            .collect();

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
                        let edge = GraphEdge {
                            source: concept.id.clone(),
                            target: target.clone(),
                            source_neighborhood: neighborhoods
                                .get(&concept.id)
                                .cloned()
                                .unwrap_or_default(),
                            target_neighborhood: neighborhoods
                                .get(target)
                                .cloned()
                                .unwrap_or_default(),
                        };
                        edges.push(edge.clone());
                        outgoing_edges
                            .entry(concept.id.clone())
                            .or_default()
                            .push(edge);
                        if neighborhoods.get(&concept.id) == neighborhoods.get(target) {
                            *local_out_degree.get_mut(&concept.id).unwrap() += 1;
                            *local_in_degree.get_mut(target).unwrap() += 1;
                        }
                    }
                }
            }
        }
        Graph {
            out_degree,
            in_degree,
            local_out_degree,
            local_in_degree,
            neighborhoods,
            edges,
            outgoing_edges,
            neighborhood_members,
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

pub fn rule_descriptors() -> Vec<RuleDescriptor> {
    rule_metas().into_iter().map(RuleDescriptor::from).collect()
}

pub fn canonical_rule_id(id: &str) -> Option<&'static str> {
    rule_metas()
        .into_iter()
        .find_map(|m| (m.id == id || m.slug() == id || m.aliases().contains(&id)).then_some(m.id))
}

pub fn rule_meta(id: &str) -> Option<RuleMeta> {
    rule_metas()
        .into_iter()
        .find(|m| m.id == id || m.slug() == id || m.aliases().contains(&id))
}

/// Inline `okf-lint-disable` rule ids per concept path.
fn inline_disables(bundle: &Bundle) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    for concept in &bundle.concepts {
        let set: HashSet<String> = match concept.frontmatter.get("okf-lint-disable") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str())
                .map(|id| canonical_rule_id(id).unwrap_or(id).to_string())
                .collect(),
            Some(Value::String(s)) => {
                std::iter::once(canonical_rule_id(s).unwrap_or(s).to_string()).collect()
            }
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
    let graph = Graph::build(bundle, config);
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
                rule_name: Some(meta.name()),
                category: Some(meta.category.as_str().to_string()),
                category_name: Some(meta.category.name().to_string()),
                severity,
                message: finding.message,
                spec: false,
                rationale: Some(meta.rationale.to_string()),
                help: Some(meta.help().to_string()),
                fix: finding.fix,
            });
        }
    }
    out
}
