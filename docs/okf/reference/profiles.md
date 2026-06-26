---
type: Concept
title: Lint profiles
description: The okf-recommended, okf-strict, and okf-minimal preset configs.
timestamp: 2026-06-23
---

# Lint profiles

A profile is a named preset selected with `extends`. okftool ships three; a
bundle inherits one and overrides what it needs. These blocks are the actual
presets compiled into okftool (kept in sync with `crates/okftool-core/presets/`
by a test).

## okf-recommended

The default — every rule at its built-in severity, overriding nothing.

```yaml
# okf-recommended — the default profile.
# Every rule runs at its built-in recommended severity (see each rule's `meta`),
# so this preset intentionally overrides nothing.
rules: {}
```

## okf-strict

Opinionated and CI-blocking: advisory rules become errors and the
off-by-default hygiene checks turn on.

```yaml
# okf-strict — opinionated, CI-blocking profile. Promotes advisory rules to
# errors and enables the off-by-default hygiene checks.
rules:
  frontmatter/require-description: error
  frontmatter/require-timestamp: warn
  frontmatter/no-empty-frontmatter-values: error
  type-vocabulary/consistent-type-casing: error
  type-vocabulary/no-singleton-type: warn
  linking/prefer-absolute-links: error
  linking/no-relative-links: error
  linking/no-dangling-links: warn
  linking/no-self-link: error
  topology/no-orphan-concepts: error
  topology/no-unindexed-concepts: error
  body/structural-body: error
  body/body-not-empty: error
  index-log/index-entry-has-description: warn
  index-log/log-newest-first: warn
```

## okf-minimal

The quietest profile: hygiene and topology advice is silenced, leaving only the
cheap format check. Spec conformance is always enforced by
[validation](/reference/validation.md), independent of any profile.

```yaml
# okf-minimal — quietest profile. Silences hygiene/topology advice; keeps only
# the cheap, high-signal format check. (Spec conformance is always enforced by
# `validate`, independent of any profile.)
rules:
  frontmatter/require-description: "off"
  frontmatter/require-timestamp: "off"
  frontmatter/no-empty-frontmatter-values: "off"
  type-vocabulary/consistent-type-casing: "off"
  type-vocabulary/no-singleton-type: "off"
  linking/prefer-absolute-links: "off"
  topology/no-orphan-concepts: "off"
  topology/no-unindexed-concepts: "off"
  topology/max-out-degree: "off"
  body/structural-body: "off"
  body/body-not-empty: "off"
  linking/no-self-link: "off"
  index-log/index-entry-has-description: "off"
  index-log/log-newest-first: "off"
  frontmatter/timestamp-iso8601: warn
```

See [configuration](/reference/configuration.md) for the full cascade and the
[lint rules](/reference/rules/index.md) each profile sets.
