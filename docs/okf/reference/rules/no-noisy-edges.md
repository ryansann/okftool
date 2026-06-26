---
type: LintRule
title: graph-structure/no-noisy-edges
description: Flags non-hub concepts with too many graph connections.
timestamp: 2026-06-26
tags: [lint, graph-structure]
---

# graph-structure/no-noisy-edges

Warns when an ordinary non-hub concept has too many incoming, outgoing, or total
graph connections.

Default options:

```yaml
rules:
  graph-structure/no-noisy-edges:
    severity: warn
    options:
      maxTotalDegree: 10
      maxOutDegree: 6
      maxInDegree: 8
```

Use this to keep ordinary concepts selective. If high edge count is intentional,
mark the concept with `hub: true`, a configured hub tag, or a configured hub
type/id.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
