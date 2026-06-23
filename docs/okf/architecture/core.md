---
type: Concept
title: okftool-core
description: The filesystem-agnostic core — parser, validator, lint engine.
timestamp: 2026-06-23
---

# okftool-core

The brain. It takes `(path, content)` pairs (no IO) so it compiles unchanged to
native and wasm.

- Parses frontmatter, bodies, links, and reserved files.
- Enforces [validation](/reference/validation.md) (spec conformance).
- Runs the lint engine, configured by [.okftool.yaml](/reference/configuration.md).

See the [overview](/architecture/overview.md).
