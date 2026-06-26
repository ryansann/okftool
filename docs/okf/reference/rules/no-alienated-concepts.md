---
type: LintRule
title: graph-structure/no-alienated-concepts
description: Flags concepts that are connected but weakly attached to their neighborhood.
timestamp: 2026-06-26
tags: [lint, graph-structure]
---

# graph-structure/no-alienated-concepts

Warns when a non-hub concept is not an orphan, but still has too little local
attachment to its neighborhood.

Default options:

```yaml
rules:
  graph-structure/no-alienated-concepts:
    severity: warn
    options:
      minLocalDegree: 1
      maxTotalDegree: 2
      minNeighborhoodSize: 2
```

This catches concepts that have one weak incoming or outgoing connection but no
real local graph context. Add a local relationship, move the concept to a better
neighborhood, or declare it as an intentional hub when appropriate.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
