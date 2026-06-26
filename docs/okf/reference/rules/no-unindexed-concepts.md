---
type: LintRule
title: topology/no-unindexed-concepts
description: A concept is not referenced by any index.md.
timestamp: 2026-06-23
tags: [lint, topology]
---

# topology/no-unindexed-concepts

A concept absent from every index is unreachable via progressive disclosure — a reader browsing the bundle will never find it.

- **Category:** topology
- **Default severity:** warn
- **Fixable:** no

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.
