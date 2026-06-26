---
type: LintRule
title: type-vocabulary/types-from-allowlist
description: A type is not in the declared vocabulary.
timestamp: 2026-06-23
tags: [lint, type-vocabulary]
---

# type-vocabulary/types-from-allowlist

Off until you declare a vocabulary via the allow option; once you do, any type outside it is almost always a mistake.

- **Category:** type-vocabulary
- **Default severity:** off
- **Fixable:** no

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.
