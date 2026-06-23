---
type: Concept
title: okftool-cli
description: The native binary surface used in CI.
timestamp: 2026-06-23
---

# okftool-cli

The `okftool` binary: `validate`, `lint`, `rules`, `explain`, `init`, and `build`.

- Emits pretty, JSON, or SARIF output.
- Exit code is driven by [.okftool.yaml](/reference/configuration.md) `ci.fail-on`.
- `build` packages a bundle into a `.tar.gz` for distribution (release artifacts).

See the [overview](/architecture/overview.md).
