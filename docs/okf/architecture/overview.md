---
type: Concept
title: Overview
description: okftool is one Rust core compiled to three surfaces.
timestamp: 2026-06-23
---

# Overview

okftool is a single Rust implementation that ships to three surfaces, so the OKF
verdict is computed once and reused everywhere.

- [okftool-core](/architecture/core.md) — the parser, validator, and lint engine
- [okftool-cli](/architecture/cli.md) — the native binary for CI
- [okftool-wasm](/architecture/wasm.md) — the npm package for JS hosts

The two layers are [validation](/reference/validation.md) and lint.
