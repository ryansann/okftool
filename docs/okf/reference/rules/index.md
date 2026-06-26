# Lint rules

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

## Body
- [body/structural-body](/reference/rules/structural-body.md) — no heading, list, or table
- [body/body-not-empty](/reference/rules/body-not-empty.md) — frontmatter but empty body

## Index & log
- [index-log/index-entry-has-description](/reference/rules/index-entry-has-description.md) — index entry lacks a description
- [index-log/log-newest-first](/reference/rules/log-newest-first.md) — log.md not newest-first
