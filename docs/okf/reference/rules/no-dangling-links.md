---
type: LintRule
title: linking/no-dangling-links
description: Flags a link that points at a concept that does not exist.
timestamp: 2026-06-23
tags: [lint, linking]
---

# linking/no-dangling-links

Broken links are explicitly tolerated by the spec (forward stubs are legitimate), so this rule is off by default — enable it for bundles that should be self-contained.

- **Category:** linking
- **Default severity:** off
- **Fixable:** no

Configured in [.okftool.yaml](/reference/configuration.md); see [validation](/reference/validation.md) for the spec layer.

Related local context: [rule taxonomy](/reference/rules/taxonomy.md).
