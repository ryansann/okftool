---
type: LintRule
title: graph-structure/require-bridge-prose
description: Flags bridging links that appear without enough surrounding prose.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/require-bridge-prose

Warns when a bridging link appears as a bare link-list item or without enough
nearby words explaining the relationship.

Default options:

```yaml
rules:
  graph-structure/require-bridge-prose:
    severity: warn
    options:
      minSurroundingWords: 6
```

OKF links are untyped. Cross-neighborhood links should carry prose that explains
why the target belongs in the reader's path.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
