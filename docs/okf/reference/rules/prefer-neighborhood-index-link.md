---
type: LintRule
title: graph-structure/prefer-neighborhood-index-link
description: Flags many deep links from one concept into another neighborhood.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/prefer-neighborhood-index-link

Warns when a non-hub concept links to several concepts in the same external
neighborhood without linking to that neighborhood's index or overview.

Default options:

```yaml
rules:
  graph-structure/prefer-neighborhood-index-link:
    severity: warn
    options:
      threshold: 3
```

Prefer one bridge to `/neighborhood/`, `/neighborhood/index.md`, or an overview
concept when a source concept needs broad access to another area.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
