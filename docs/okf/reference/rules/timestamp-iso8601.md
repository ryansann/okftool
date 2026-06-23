---
type: LintRule
title: timestamp-iso8601
description: Flags a timestamp that is not ISO-8601.
timestamp: 2026-06-23
tags: [lint, frontmatter]
---

# timestamp-iso8601

Non-ISO timestamps sort lexically wrong and are ambiguous across locales.

- **Category:** frontmatter
- **Default severity:** error
- **Fixable:** yes

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.
