---
type: Concept
title: Validation
description: The spec (section 9) conformance layer — non-disableable errors.
timestamp: 2026-06-23
---

# Validation

`validate` enforces only what the OKF spec says MUST hold:

- every non-reserved file has a parseable frontmatter block;
- every such block has a non-empty `type`.

These are non-disableable errors. Everything advisory lives in
[configuration](/reference/configuration.md) as [lint rules](/reference/rules/index.md).
