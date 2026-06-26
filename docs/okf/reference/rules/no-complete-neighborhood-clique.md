---
type: LintRule
title: graph-structure/no-complete-neighborhood-clique
description: Flags neighborhoods whose local link density is too high.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/no-complete-neighborhood-clique

Warns when a neighborhood has enough concepts and too many possible directed
local links are present.

Default options:

```yaml
rules:
  graph-structure/no-complete-neighborhood-clique:
    severity: warn
    options:
      maxDensity: 0.45
      minNodes: 5
```

Dense local cliques make each edge less meaningful. Use selective local links so
the graph preserves signal.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
