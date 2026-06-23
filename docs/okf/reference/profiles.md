---
type: Concept
title: Lint profiles
description: The okf-recommended, okf-strict, and okf-minimal preset configs.
timestamp: 2026-06-23
---

# Lint profiles

A profile is a named preset selected with `extends`. okflint ships three; a
bundle inherits one and overrides what it needs. These blocks are the actual
presets compiled into okflint (kept in sync with `presets/` by a test).

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
  consistent-type-casing: error
  prefer-absolute-links: error
  no-orphan-concepts: error
  structural-body: error
  no-dangling-links: warn
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
  consistent-type-casing: "off"
  prefer-absolute-links: "off"
  no-orphan-concepts: "off"
  max-out-degree: "off"
  structural-body: "off"
  no-dangling-links: "off"
  timestamp-iso8601: warn
```

See [configuration](/reference/configuration.md) for the full cascade and the
[lint rules](/reference/rules/index.md) each profile sets.
