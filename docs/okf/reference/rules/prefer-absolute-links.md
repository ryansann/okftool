---
type: LintRule
title: linking/prefer-absolute-links
description: Flags an internal link that is relative rather than bundle-absolute.
timestamp: 2026-06-23
tags: [lint, linking]
---

# linking/prefer-absolute-links

Bundle-absolute links survive moving or renaming the source file; relative links break.

- **Category:** linking
- **Default severity:** warn
- **Fixable:** yes

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
