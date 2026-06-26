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
- Powers both the [CLI](/architecture/cli.md) and [wasm package](/architecture/wasm.md).
- Enforces the spec conformance layer through [validation](/reference/validation.md), so parse errors stay separate from advisory lint.
- Runs the lint engine with [.okftool.yaml](/reference/configuration.md), allowing bundles to tune profile, rule severity, and graph neighborhoods.

See the [overview](/architecture/overview.md).
