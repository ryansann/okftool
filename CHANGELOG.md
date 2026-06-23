# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-06-23

### Added

- `okftool-core`: OKF parser, §9 `validate` (conformance), and a configurable
  lint engine.
- 18 lint rules across six categories (frontmatter, type-vocabulary, linking,
  topology, body, index/log).
- `.okftool.yaml` configuration: `extends` presets (okf-recommended / okf-strict
  / okf-minimal), per-rule severity and options, glob `overrides`, inline
  `okf-lint-disable`, and a `ci.fail-on` gate.
- `okftool` CLI: `validate`, `lint`, `rules`, `explain`, `init`, and `build`
  (package a bundle as a reproducible `.tar.gz`), with
  `--format pretty|json|sarif`.
- Releases attach the packaged `docs/okf` bundle as `okftool-<version>.tar.gz`.
- `okftool-wasm`: WebAssembly bindings published as an npm package, with
  generated TypeScript types.
- Self-documenting OKF bundle at `docs/okf/`, linted under the strict profile in
  CI (dogfood).

[Unreleased]: https://github.com/ryansann/okftool/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/ryansann/okftool/releases/tag/v0.1.2
