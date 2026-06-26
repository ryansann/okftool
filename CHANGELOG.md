# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5] - 2026-06-26

### Added

- Added `graph-structure/no-alienated-concepts` to flag concepts that are
  connected but weakly attached to their local neighborhood.
- Added `graph-structure/no-noisy-edges` to flag ordinary concepts with too many
  incoming, outgoing, or total graph connections.

## [0.2.3] - 2026-06-25

### Fixed

- Fixed the crates.io release preflight to check exact crate versions via the
  crates.io API instead of `cargo info`, which could treat an existing crate as
  an existing version.

## [0.2.2] - 2026-06-25

### Fixed

- Made crates.io publishing retry-safe after partial release success and switched
  crates.io propagation checks to exact `cargo info crate@version` lookups.
- Updated GitHub Actions workflow action versions to Node 24-compatible majors.

## [0.2.1] - 2026-06-25

### Fixed

- Made `okftool-wasm` package metadata explicit so `wasm-pack` can parse the
  manifest during npm package builds.

## [0.2.0] - 2026-06-25

### Added

- Added canonical namespaced lint rule IDs, alias-aware config resolution, and
  richer diagnostic/rule metadata for okfview.
- Added graph-structure rules for neighborhood cohesion, bridge fanout, bridge
  prose, dense local cliques, and explicit hub declaration.
- Published a dual-target npm package with browser/Vite and Node/Electron-main
  entrypoints.

## [0.1.5] - 2026-06-23

### Fixed

- Published the wasm package under the scoped npm name `@ryansann/okftool`
  because npm blocks the unscoped `okftool` name as too similar to `okf-tool`.

## [0.1.4] - 2026-06-23

### Fixed

- Normalized the generated npm package repository URL before packing and
  publishing.

## [0.1.3] - 2026-06-23

### Fixed

- Fixed npm publishing in the release workflow by publishing from the
  `wasm-pack` output directory.
- Updated the Intel macOS release runner label to a currently available
  GitHub-hosted runner.
- Made `okftool-core` self-contained for crates.io packaging by moving embedded
  presets into the crate.
- Included `README.md` and `LICENSE` in the generated npm package.

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

[Unreleased]: https://github.com/ryansann/okftool/compare/v0.2.5...HEAD
[0.2.5]: https://github.com/ryansann/okftool/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/ryansann/okftool/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ryansann/okftool/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ryansann/okftool/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ryansann/okftool/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ryansann/okftool/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/ryansann/okftool/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/ryansann/okftool/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ryansann/okftool/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ryansann/okftool/releases/tag/v0.1.2
