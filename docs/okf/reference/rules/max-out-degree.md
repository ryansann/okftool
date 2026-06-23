---
type: LintRule
title: max-out-degree
description: Flags a concept that exceeds the out-degree cap.
timestamp: 2026-06-23
tags: [lint, topology]
---

# max-out-degree

A concept linking to dozens of others is usually an undeclared hub; cap it with the max option or exempt hub types.

- **Category:** topology
- **Default severity:** warn
- **Fixable:** no

Configured in [.okflint.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.
