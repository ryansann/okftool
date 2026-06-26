# Lint rules

For how these rules are grouped and named, see the local
[rule taxonomy](/reference/rules/taxonomy.md).

- [Rule taxonomy](/reference/rules/taxonomy.md) — how lint rules are grouped and named

## Frontmatter
- [frontmatter/require-description](/reference/rules/require-description.md) — concept has no description
- [frontmatter/require-timestamp](/reference/rules/require-timestamp.md) — concept has no timestamp
- [frontmatter/timestamp-iso8601](/reference/rules/timestamp-iso8601.md) — timestamp is not ISO-8601
- [frontmatter/no-empty-frontmatter-values](/reference/rules/no-empty-frontmatter-values.md) — title/description present but empty

## Type vocabulary
- [type-vocabulary/consistent-type-casing](/reference/rules/consistent-type-casing.md) — a type differs only by case
- [type-vocabulary/no-singleton-type](/reference/rules/no-singleton-type.md) — a type used by exactly one concept
- [type-vocabulary/types-from-allowlist](/reference/rules/types-from-allowlist.md) — a type outside the declared vocabulary

## Linking
- [linking/prefer-absolute-links](/reference/rules/prefer-absolute-links.md) — internal link is relative
- [linking/no-relative-links](/reference/rules/no-relative-links.md) — any relative internal link (stricter)
- [linking/no-dangling-links](/reference/rules/no-dangling-links.md) — link to a nonexistent concept
- [linking/no-self-link](/reference/rules/no-self-link.md) — a concept links to itself

## Topology
- [topology/no-orphan-concepts](/reference/rules/no-orphan-concepts.md) — no incoming or outgoing links
- [topology/no-unindexed-concepts](/reference/rules/no-unindexed-concepts.md) — not referenced by any index
- [topology/max-out-degree](/reference/rules/max-out-degree.md) — exceeds the out-degree cap

## Graph structure
- [graph-structure/no-excessive-bridging](/reference/rules/no-excessive-bridging.md) — too many cross-neighborhood links
- [graph-structure/bridging-ratio](/reference/rules/bridging-ratio.md) — outgoing links mostly leave the neighborhood
- [graph-structure/no-leaf-bridge-fanout](/reference/rules/no-leaf-bridge-fanout.md) — leaf bridges into too many neighborhoods
- [graph-structure/require-bridge-prose](/reference/rules/require-bridge-prose.md) — bridging link lacks enough explanatory prose
- [graph-structure/prefer-neighborhood-index-link](/reference/rules/prefer-neighborhood-index-link.md) — many deep links to one external neighborhood
- [graph-structure/no-complete-neighborhood-clique](/reference/rules/no-complete-neighborhood-clique.md) — local neighborhood is too densely linked
- [graph-structure/min-local-cohesion](/reference/rules/min-local-cohesion.md) — outgoing links but no local cohesive edge
- [graph-structure/declare-hubs](/reference/rules/declare-hubs.md) — high-fanout concept is not declared as a hub

## Body
- [body/structural-body](/reference/rules/structural-body.md) — no heading, list, or table
- [body/body-not-empty](/reference/rules/body-not-empty.md) — frontmatter but empty body

## Index & log
- [index-log/index-entry-has-description](/reference/rules/index-entry-has-description.md) — index entry lacks a description
- [index-log/log-newest-first](/reference/rules/log-newest-first.md) — log.md not newest-first
