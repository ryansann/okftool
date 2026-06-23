---
type: Concept
title: Configuration
description: The .okflint.yaml format — presets, rules, overrides, CI gate.
timestamp: 2026-06-23
---

# Configuration

`.okflint.yaml` resolves a cascade: preset then root rules then path overrides
then inline disables.

- `extends` pulls a [profile](/reference/profiles.md) (okf-recommended, okf-strict, okf-minimal).
- `rules` set per-rule severity and options.
- `overrides` scope rules by glob; `ci.fail-on` gates exit.

Concepts can opt out inline with an `okf-lint-disable` frontmatter list. See
[validation](/reference/validation.md) and the [lint rules](/reference/rules/index.md).
