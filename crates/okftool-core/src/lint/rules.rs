//! The rule catalog. Each rule is a unit struct; `registry()` lists them in
//! display order. Adding a rule = add a struct + one `registry()` line.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;
use serde_json::Value;

use super::{Category, Finding, LintContext, Rule, RuleMeta};
use crate::links::{is_external, resolve_target};
use crate::model::Severity;

pub fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        // frontmatter
        Box::new(RequireDescription),
        Box::new(RequireTimestamp),
        Box::new(TimestampIso8601),
        Box::new(NoEmptyFrontmatterValues),
        // type vocabulary
        Box::new(ConsistentTypeCasing),
        Box::new(NoSingletonType),
        Box::new(TypesFromAllowlist),
        // linking
        Box::new(PreferAbsoluteLinks),
        Box::new(NoRelativeLinks),
        Box::new(NoDanglingLinks),
        Box::new(NoSelfLink),
        // topology
        Box::new(NoOrphanConcepts),
        Box::new(NoUnindexedConcepts),
        Box::new(MaxOutDegree),
        // body
        Box::new(StructuralBody),
        Box::new(BodyNotEmpty),
        // index & log
        Box::new(IndexEntryHasDescription),
        Box::new(LogNewestFirst),
    ]
}

// ---- frontmatter ---------------------------------------------------------------

struct RequireDescription;
impl Rule for RequireDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "frontmatter/require-description",
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
            id: "frontmatter/require-timestamp",
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
            id: "frontmatter/timestamp-iso8601",
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
            id: "type-vocabulary/consistent-type-casing",
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
            id: "linking/prefer-absolute-links",
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
            id: "linking/no-dangling-links",
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
            id: "topology/no-orphan-concepts",
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
            id: "topology/max-out-degree",
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
            id: "body/structural-body",
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

struct BodyNotEmpty;
impl Rule for BodyNotEmpty {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "body/body-not-empty",
            category: Category::Body,
            summary: "Concept has frontmatter but an empty body.",
            rationale: "A frontmatter-only stub is fine to commit, but worth flagging — it carries no prose for retrieval.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| !c.frontmatter.is_empty() && c.body.trim().is_empty())
            .map(|c| Finding::new(&c.path, "Concept has frontmatter but no body."))
            .collect()
    }
}

// ---- additional frontmatter ----------------------------------------------------

struct NoEmptyFrontmatterValues;
impl Rule for NoEmptyFrontmatterValues {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "frontmatter/no-empty-frontmatter-values",
            category: Category::Frontmatter,
            summary: "A frontmatter `title`/`description` is present but empty.",
            rationale: "An empty string reads as 'documented' to tooling while conveying nothing; omit the key or fill it in.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in &ctx.bundle.concepts {
            for key in ["title", "description"] {
                if let Some(Value::String(s)) = c.frontmatter.get(key) {
                    if s.trim().is_empty() {
                        out.push(Finding::new(
                            &c.path,
                            format!("Frontmatter `{key}` is present but empty."),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ---- additional type vocabulary ------------------------------------------------

struct NoSingletonType;
impl Rule for NoSingletonType {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "type-vocabulary/no-singleton-type",
            category: Category::TypeVocabulary,
            summary: "A `type` is used by exactly one concept.",
            rationale: "A type with a single member is often a typo or over-specialization that fragments the vocabulary.",
            default_severity: Severity::Info,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for c in &ctx.bundle.concepts {
            if let Some(t) = c.concept_type.as_deref() {
                *counts.entry(t).or_default() += 1;
            }
        }
        ctx.bundle
            .concepts
            .iter()
            .filter_map(|c| {
                let t = c.concept_type.as_deref()?;
                (counts.get(t).copied().unwrap_or(0) == 1).then(|| {
                    Finding::new(&c.path, format!("Type `{t}` is used by only this concept."))
                })
            })
            .collect()
    }
}

struct TypesFromAllowlist;
impl Rule for TypesFromAllowlist {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "type-vocabulary/types-from-allowlist",
            category: Category::TypeVocabulary,
            summary: "A `type` is not in the declared vocabulary.",
            rationale: "Off until you declare a vocabulary; once you do (`options.allow`), any type outside it is almost always a mistake. Options: `allow` (list).",
            default_severity: Severity::Off,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let allow: HashSet<&str> = ctx
            .options
            .get("allow")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
            .unwrap_or_default();
        if allow.is_empty() {
            return Vec::new(); // no vocabulary declared → nothing to enforce
        }
        ctx.bundle
            .concepts
            .iter()
            .filter_map(|c| {
                let t = c.concept_type.as_deref()?;
                (!allow.contains(t)).then(|| {
                    Finding::new(
                        &c.path,
                        format!("Type `{t}` is not in the declared vocabulary."),
                    )
                })
            })
            .collect()
    }
}

// ---- additional linking --------------------------------------------------------

struct NoRelativeLinks;
impl Rule for NoRelativeLinks {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "linking/no-relative-links",
            category: Category::Linking,
            summary: "Any relative internal link (stricter than prefer-absolute-links).",
            rationale: "A blanket ban on relative links for bundles that want every cross-link to survive refactors.",
            default_severity: Severity::Off,
            fixable: true,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for c in &ctx.bundle.concepts {
            for link in &c.outgoing {
                if !is_external(&link.href)
                    && !link.href.starts_with('/')
                    && !link.href.starts_with('#')
                    && !link.href.ends_with('/')
                {
                    out.push(Finding::with_fix(
                        &c.path,
                        format!(
                            "Relative link `{}` — use a bundle-absolute `/…` path.",
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

struct NoSelfLink;
impl Rule for NoSelfLink {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "linking/no-self-link",
            category: Category::Linking,
            summary: "A concept links to itself.",
            rationale: "A self-link adds a noise edge to the graph and conveys nothing.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| {
                c.outgoing
                    .iter()
                    .any(|l| l.target_id.as_deref() == Some(c.id.as_str()))
            })
            .map(|c| Finding::new(&c.path, "Concept links to itself."))
            .collect()
    }
}

// ---- additional topology -------------------------------------------------------

struct NoUnindexedConcepts;
impl Rule for NoUnindexedConcepts {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "topology/no-unindexed-concepts",
            category: Category::Topology,
            summary: "Concept is not referenced by any index.md.",
            rationale: "A concept absent from every index is unreachable via progressive disclosure — a reader browsing the bundle will never find it.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        // No index system → nothing can be "unindexed".
        if ctx.bundle.indexes.is_empty() {
            return Vec::new();
        }
        let mut indexed: HashSet<String> = HashSet::new();
        for index in &ctx.bundle.indexes {
            let from = if index.dir.is_empty() {
                "index".to_string()
            } else {
                format!("{}/index", index.dir)
            };
            for section in &index.sections {
                for entry in &section.entries {
                    if let Some(target) = resolve_target(&from, &entry.href).target_path {
                        indexed.insert(target);
                    }
                }
            }
        }
        ctx.bundle
            .concepts
            .iter()
            .filter(|c| !indexed.contains(&c.id))
            .map(|c| Finding::new(&c.path, "Concept is not referenced by any index.md."))
            .collect()
    }
}

// ---- index & log ---------------------------------------------------------------

struct IndexEntryHasDescription;
impl Rule for IndexEntryHasDescription {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "index-log/index-entry-has-description",
            category: Category::IndexLog,
            summary: "An index entry has no `— description` tail.",
            rationale: "Descriptions in an index are the snippets a reader scans; entries without one make progressive disclosure poorer.",
            default_severity: Severity::Info,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        let mut out = Vec::new();
        for index in &ctx.bundle.indexes {
            for section in &index.sections {
                for entry in &section.entries {
                    if entry.description.is_none() {
                        out.push(Finding::new(
                            &index.path,
                            format!("Index entry `{}` has no description.", entry.title),
                        ));
                    }
                }
            }
        }
        out
    }
}

struct LogNewestFirst;
impl Rule for LogNewestFirst {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: "index-log/log-newest-first",
            category: Category::IndexLog,
            summary: "log.md entries are not in newest-first date order.",
            rationale: "A change log should read top-down newest-first; out-of-order dates suggest an append in the wrong place.",
            default_severity: Severity::Warn,
            fixable: false,
        }
    }
    fn check(&self, ctx: &LintContext) -> Vec<Finding> {
        // ISO dates compare lexically, so descending order == newest-first.
        ctx.bundle
            .logs
            .iter()
            .filter(|log| log.days.windows(2).any(|w| w[0].date < w[1].date))
            .map(|log| Finding::new(&log.path, "log.md entries are not in newest-first order."))
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
