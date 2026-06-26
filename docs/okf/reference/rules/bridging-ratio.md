---
type: LintRule
title: graph-structure/bridging-ratio
description: Flags concepts whose outgoing links mostly leave their neighborhood.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/bridging-ratio

Warns when a non-hub concept has enough outgoing links to evaluate and too large
a share of those links point outside its neighborhood.

Default options:

```yaml
rules:
  graph-structure/bridging-ratio:
    severity: warn
    options:
      maxRatio: 0.4
      minOutDegree: 3
```

A concept with one bridge among several local links is fine. A concept whose
links mostly route elsewhere probably needs local context, a new home, or a
declared overview concept.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
