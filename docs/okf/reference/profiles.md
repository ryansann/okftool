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
  require-description: error
  require-timestamp: warn
  no-empty-frontmatter-values: error
  consistent-type-casing: error
  no-singleton-type: warn
  prefer-absolute-links: error
  no-relative-links: error
  no-dangling-links: warn
  no-self-link: error
  no-orphan-concepts: error
  no-unindexed-concepts: error
  structural-body: error
  body-not-empty: error
  index-entry-has-description: warn
  log-newest-first: warn
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
  require-description: "off"
  require-timestamp: "off"
  no-empty-frontmatter-values: "off"
  consistent-type-casing: "off"
  no-singleton-type: "off"
  prefer-absolute-links: "off"
  no-orphan-concepts: "off"
  no-unindexed-concepts: "off"
  max-out-degree: "off"
  structural-body: "off"
  body-not-empty: "off"
  no-self-link: "off"
  index-entry-has-description: "off"
  log-newest-first: "off"
  timestamp-iso8601: warn
```

See [configuration](/reference/configuration.md) for the full cascade and the
[lint rules](/reference/rules/index.md) each profile sets.
