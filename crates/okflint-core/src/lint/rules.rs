//! The rule catalog. Each rule is a unit struct; `registry()` lists them in
//! display order. Adding a rule = add a struct + one `registry()` line.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;

use super::{Category, Finding, LintContext, Rule, RuleMeta};
use crate::links::is_external;
use crate::model::Severity;

pub fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(RequireDescription),
        Box::new(RequireTimestamp),
        Box::new(TimestampIso8601),
        Box::new(ConsistentTypeCasing),
        Box::new(PreferAbsoluteLinks),
        Box::new(NoDanglingLinks),
        Box::new(NoOrphanConcepts),
        Box::new(MaxOutDegree),
        Box::new(StructuralBody),
    ]
}

// ---- frontmatter ---------------------------------------------------------------

struct RequireDescription;
impl Rule for RequireDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "require-description",
            category: Category::Frontmatter,
            summary: "Concept has no `description`.",
            rationale: "The description is the unit of progressive disclosure — it is what index snippets and search results show.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| {
                c.description
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
            })
            .map(|c| Finding::new(&c.path, "Concept has no `description`."))
            .collect()
    }
}

struct RequireTimestamp;
impl Rule for RequireTimestamp {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "require-timestamp",
            category: Category::Frontmatter,
            summary: "Concept has no `timestamp`.",
            rationale:
                "A timestamp enables sort-by-recency and audit of when knowledge last changed.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| {
                c.timestamp
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or("")
                    .is_empty()
            })
            .map(|c| Finding::new(&c.path, "Concept has no `timestamp`."))
            .collect()
    }
}

struct TimestampIso8601;
impl Rule for TimestampIso8601 {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "timestamp-iso8601",
            category: Category::Frontmatter,
            summary: "`timestamp` is present but not ISO-8601.",
            rationale: "Non-ISO timestamps sort lexically wrong and are ambiguous across locales.",
            default_severity: Severity::Error,
            fixable: true,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            Regex::new(
                r"^\d{4}-\d{2}-\d{2}([Tt ]\d{2}:\d{2}(:\d{2})?(\.\d+)?([Zz]|[+-]\d{2}:?\d{2})?)?$",
            )
            .unwrap()
        });
        ctx.bundle
            .concepts
            .iter()
            .filter_map(|c| c.timestamp.as_deref().map(|t| (c, t)))
            .filter(|(_, t)| !t.trim().is_empty() && !re.is_match(t.trim()))
            .map(|(c, t)| {
                Finding::with_fix(
                    &c.path,
                    format!("`timestamp` \"{t}\" is not ISO-8601."),
                    "Reformat as YYYY-MM-DD or an ISO-8601 datetime.",
                )
            })
            .collect()
    }
}

// ---- type vocabulary -----------------------------------------------------------

struct ConsistentTypeCasing;
impl Rule for ConsistentTypeCasing {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "consistent-type-casing",
            category: Category::TypeVocabulary,
            summary: "A `type` differs from another only by case.",
            rationale:
                "`Table` vs `table` fragment the type vocabulary and split filters and the graph.",
            default_severity: Severity::Warn,
            fixable: true,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        // Group distinct types by case-insensitive key.
        let mut groups: HashMap<String, Vec<&str>> = HashMap::new();
        for t in &ctx.bundle.types {
            groups.entry(t.to_lowercase()).or_default().push(t);
        }
        let mut out = Vec::new();
        for concept in &ctx.bundle.concepts {
            let Some(t) = concept.concept_type.as_deref() else {
                continue;
            };
            if let Some(variants) = groups.get(&t.to_lowercase()) {
                if variants.len() > 1 {
                    let others: Vec<&str> = variants.iter().copied().filter(|v| *v != t).collect();
                    out.push(Finding::with_fix(
                        &concept.path,
                        format!("Type `{t}` differs only in case from {}.", quoted(&others)),
                        "Normalize to a single canonical casing across the bundle.",
                    ));
                }
            }
        }
        out
    }
}

// ---- linking -------------------------------------------------------------------

struct PreferAbsoluteLinks;
impl Rule for PreferAbsoluteLinks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "prefer-absolute-links",
            category: Category::Linking,
            summary: "Internal link is relative, not bundle-absolute.",
            rationale: "Bundle-absolute `/x.md` links survive moving or renaming the source file; relative links break.",
            default_severity: Severity::Warn,
            fixable: true,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for concept in &ctx.bundle.concepts {
            for link in &concept.outgoing {
                let internal = link.target_id.is_some() || link.broken;
                let relative = !is_external(&link.href)
                    && !link.href.starts_with('/')
                    && !link.href.starts_with('#');
                if internal && relative {
                    out.push(Finding::with_fix(
                        &concept.path,
                        format!(
                            "Relative internal link `{}` — prefer a bundle-absolute `/…` path.",
                            link.href
                        ),
                        "Rewrite as a bundle-absolute link beginning with `/`.",
                    ));
                }
            }
        }
        out
    }
}

struct NoDanglingLinks;
impl Rule for NoDanglingLinks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "no-dangling-links",
            category: Category::Linking,
            summary: "Link points at a concept that does not exist.",
            rationale: "Broken links are explicitly tolerated by the spec (forward stubs are legitimate), so this is off by default — enable it for bundles that should be self-contained.",
            default_severity: Severity::Off,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for concept in &ctx.bundle.concepts {
            for link in &concept.outgoing {
                if link.broken {
                    out.push(Finding::new(
                        &concept.path,
                        format!(
                            "Link `{}` has no matching concept in the bundle.",
                            link.href
                        ),
                    ));
                }
            }
        }
        out
    }
}

// ---- topology ------------------------------------------------------------------

struct NoOrphanConcepts;
impl Rule for NoOrphanConcepts {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "no-orphan-concepts",
            category: Category::Topology,
            summary: "Concept has no incoming or outgoing links.",
            rationale: "A degree-0 concept is unreachable by graph traversal and contributes nothing to the knowledge graph.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| {
                ctx.graph.out_degree.get(&c.id).copied().unwrap_or(0) == 0
                    && ctx.graph.in_degree.get(&c.id).copied().unwrap_or(0) == 0
            })
            .map(|c| {
                Finding::new(
                    &c.path,
                    "Concept is an orphan (no incoming or outgoing links).",
                )
            })
            .collect()
    }
}

struct MaxOutDegree;
impl Rule for MaxOutDegree {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "max-out-degree",
            category: Category::Topology,
            summary: "Concept exceeds the out-degree cap (hairball guard).",
            rationale: "A concept linking to dozens of others is usually an undeclared hub; cap it or mark it an exempt hub type. Options: `max` (default 20), `exempt-types`.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let max = ctx
            .options
            .get("max")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize;
        let exempt: HashSet<String> = ctx
            .options
            .get("exempt-types")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| {
                !c.concept_type
                    .as_deref()
                    .is_some_and(|t| exempt.contains(t))
            })
            .filter_map(|c| {
                let deg = ctx.graph.out_degree.get(&c.id).copied().unwrap_or(0);
                (deg > max).then(|| {
                    Finding::new(
                        &c.path,
                        format!("Concept links out to {deg} concepts (cap {max})."),
                    )
                })
            })
            .collect()
    }
}

// ---- body ----------------------------------------------------------------------

struct StructuralBody;
impl Rule for StructuralBody {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "structural-body",
            category: Category::Body,
            summary: "Body is an unstructured prose wall (no heading/list/table).",
            rationale: "Headings, lists, and tables retrieve far better than a wall of prose and give agents anchors to cite.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        static RE: OnceLock<Regex> = OnceLock::new();
        // A heading, a bullet/numbered list item, or a table row.
        let re = RE.get_or_init(|| Regex::new(r"(?m)^\s*(#{1,6}\s|[-*+]\s|\d+\.\s|\|)").unwrap());
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| !c.body.trim().is_empty() && !re.is_match(&c.body))
            .map(|c| {
                Finding::new(
                    &c.path,
                    "Body has no headings, lists, or tables — it will retrieve poorly.",
                )
            })
            .collect()
    }
}

fn quoted(items: &[&str]) -> String {
    items
        .iter()
        .map(|s| format!("`{s}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
