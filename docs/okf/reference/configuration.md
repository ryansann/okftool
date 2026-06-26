---
type: Concept
title: Configuration
description: The .okftool.yaml format — presets, rules, overrides, CI gate.
timestamp: 2026-06-23
---

# Configuration

`.okftool.yaml` resolves a cascade: preset then root rules then path overrides
then inline disables.

- `extends` pulls a [profile](/reference/profiles.md) (okf-recommended, okf-strict, okf-minimal).
- `rules` set per-rule severity and options.
- `graph.neighborhoods` can declare conceptual neighborhoods for graph-structure
  rules. Configured exact paths and globs win over concept frontmatter, which
  wins over directory/root inference.
- `overrides` scope rules by glob; `ci.fail-on` gates exit.

Concepts can opt out inline with an `okf-lint-disable` frontmatter list. See
[validation](/reference/validation.md) and the [lint rules](/reference/rules/index.md).

Rule IDs are namespaced by category, for example
`frontmatter/require-description` and `topology/no-orphan-concepts`. Older flat
IDs such as `require-description` remain supported as aliases for configuration,
overrides, inline disables, and `okftool explain`.

```yaml
graph:
  neighborhoods:
    graph-authoring:
      paths:
        - principles/local-neighborhoods.md
        - rules/graph-coherence.md
    reference:
      paths:
        - reference/*.md
        - reference/rules/*.md
```
