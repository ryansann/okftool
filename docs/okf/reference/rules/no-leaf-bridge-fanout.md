---
type: LintRule
title: graph-structure/no-leaf-bridge-fanout
description: Flags leaf concepts that bridge into too many target neighborhoods.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/no-leaf-bridge-fanout

Warns when an ordinary non-hub concept links into more than one target
neighborhood.

Default options:

```yaml
rules:
  graph-structure/no-leaf-bridge-fanout:
    severity: warn
    options:
      maxTargetNeighborhoods: 1
```

If a concept needs to connect several neighborhoods, extract or declare a hub,
overview, map, or reference concept.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
