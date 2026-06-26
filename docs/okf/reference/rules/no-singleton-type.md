---
type: LintRule
title: type-vocabulary/no-singleton-type
description: A type is used by exactly one concept.
timestamp: 2026-06-23
tags: [lint, type-vocabulary]
---

# type-vocabulary/no-singleton-type

A type with a single member is often a typo or over-specialization that fragments the vocabulary.

- **Category:** type-vocabulary
- **Default severity:** info
- **Fixable:** no

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
