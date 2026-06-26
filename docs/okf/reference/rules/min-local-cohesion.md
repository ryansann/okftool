---
type: LintRule
title: graph-structure/min-local-cohesion
description: Flags concepts with outgoing links but no local cohesive edge.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/min-local-cohesion

Warns when a non-hub concept has outgoing links but none stay inside its
neighborhood.

Default options:

```yaml
rules:
  graph-structure/min-local-cohesion:
    severity: warn
    options:
      requireLocalEdge: true
```

A concept should visibly belong somewhere. If every outgoing edge points
elsewhere, add local context or move the concept to a better neighborhood.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
