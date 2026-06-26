---
type: LintRule
title: graph-structure/declare-hubs
description: Flags high-fanout concepts that are not declared as hubs.
timestamp: 2026-06-25
tags: [lint, graph-structure]
---

# graph-structure/declare-hubs

Warns when a concept links out to many other concepts but is not marked as an
intentional hub.

Default options:

```yaml
rules:
  graph-structure/declare-hubs:
    severity: warn
    options:
      outDegree: 8
      hubTags: [hub, overview, map]
```

Mark intentional routers with `hub: true`, a hub tag, a configured hub id, or a
configured hub type. This lets leaf-shape rules stay strict without punishing
real overview concepts.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
