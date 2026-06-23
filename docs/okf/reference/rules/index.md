# Lint rules

## Frontmatter
- [require-description](/reference/rules/require-description.md) — concept has no description
- [require-timestamp](/reference/rules/require-timestamp.md) — concept has no timestamp
- [timestamp-iso8601](/reference/rules/timestamp-iso8601.md) — timestamp is not ISO-8601
- [no-empty-frontmatter-values](/reference/rules/no-empty-frontmatter-values.md) — title/description present but empty

## Type vocabulary
- [consistent-type-casing](/reference/rules/consistent-type-casing.md) — a type differs only by case
- [no-singleton-type](/reference/rules/no-singleton-type.md) — a type used by exactly one concept
- [types-from-allowlist](/reference/rules/types-from-allowlist.md) — a type outside the declared vocabulary

## Linking
- [prefer-absolute-links](/reference/rules/prefer-absolute-links.md) — internal link is relative
- [no-relative-links](/reference/rules/no-relative-links.md) — any relative internal link (stricter)
- [no-dangling-links](/reference/rules/no-dangling-links.md) — link to a nonexistent concept
- [no-self-link](/reference/rules/no-self-link.md) — a concept links to itself

## Topology
- [no-orphan-concepts](/reference/rules/no-orphan-concepts.md) — no incoming or outgoing links
- [no-unindexed-concepts](/reference/rules/no-unindexed-concepts.md) — not referenced by any index
- [max-out-degree](/reference/rules/max-out-degree.md) — exceeds the out-degree cap

## Body
- [structural-body](/reference/rules/structural-body.md) — no heading, list, or table
- [body-not-empty](/reference/rules/body-not-empty.md) — frontmatter but empty body

## Index & log
- [index-entry-has-description](/reference/rules/index-entry-has-description.md) — index entry lacks a description
- [log-newest-first](/reference/rules/log-newest-first.md) — log.md not newest-first
