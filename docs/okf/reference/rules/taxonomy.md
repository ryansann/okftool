---
type: Concept
title: Rule taxonomy
description: How okftool lint rules are grouped and named.
timestamp: 2026-06-25
tags: [lint, hub, overview]
hub: true
---

# Rule taxonomy

okftool rule IDs are namespaced as `category/rule-name`. Categories let the CLI,
SARIF output, wasm API, and okfview group diagnostics without guessing from
message text.

- Frontmatter rules describe metadata quality.
- Type vocabulary rules keep `type` values coherent.
- Linking rules evaluate individual markdown links.
- Topology rules evaluate basic graph health.
- Graph-structure rules evaluate neighborhood cohesion and bridge shape.
- Body rules evaluate retrieval-friendly concept bodies.
- Index & log rules evaluate reserved `index.md` and `log.md` files.

See the [rule catalog](/reference/rules/index.md) for the complete list.
