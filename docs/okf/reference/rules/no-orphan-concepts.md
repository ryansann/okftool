---
type: LintRule
title: topology/no-orphan-concepts
description: Flags a concept with no incoming or outgoing links.
timestamp: 2026-06-23
tags: [lint, topology]
---

# topology/no-orphan-concepts

A degree-zero concept is unreachable by graph traversal and contributes nothing to the knowledge graph.

- **Category:** topology
- **Default severity:** warn
- **Fixable:** no

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
