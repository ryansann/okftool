---
type: LintRule
title: graph-structure/no-excessive-bridging
description: Flags ordinary concepts with too many outgoing cross-neighborhood links.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/no-excessive-bridging

Warns when a non-hub concept has more outgoing bridging edges than the allowed
maximum.

Default options:

```yaml
rules:
  graph-structure/no-excessive-bridging:
    severity: warn
    options:
      max: 2
```

Declare intentional routers with `hub: true`, a hub tag, a configured hub id, or
a configured hub type.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
