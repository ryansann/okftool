# okftool

[![CI](https://github.com/ryansann/okftool/actions/workflows/ci.yml/badge.svg)](https://github.com/ryansann/okftool/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](./LICENSE)

A fast, embeddable **validator and linter** for [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
(Open Knowledge Format) bundles, written in Rust.

One core, three surfaces:

| Surface | Crate | Use |
|---|---|---|
| **Native CLI** | `okftool-cli` → `okftool` | CI, local `okftool validate` / `okftool lint` |
| **wasm / npm** | `okftool-wasm` → `@ryansann/okftool` npm package | embed in JS hosts (a desktop app, a Node/edge API, the browser) |
| **Rust crate** | `okftool-core` | embed directly in a Rust host |

The brain — parsing, spec validation, and the lint engine — lives in `okftool-core`
and is filesystem-agnostic (callers pass `(path, content)`), so it compiles
unchanged to native and `wasm32`. There is exactly one implementation.

## Two layers

okftool mirrors the OKF spec's MUST/SHOULD split:

- **`validate`** enforces what the spec says MUST be true (§9 conformance:
  parseable frontmatter, a non-empty `type`). These are non-disableable errors.
- **`lint`** enforces what SHOULD be true plus everything the spec leaves to
  judgment. Every lint rule is configurable and disableable. A lint rule may
  never flag something the spec mandates tolerating (broken links, unknown
  `type` values, missing optional fields) as a non-disableable error.

## Install

```sh
# from source (requires a Rust toolchain)
cargo install --path crates/okftool-cli
# or, once published
cargo install okftool-cli
```

Prebuilt binaries for macOS/Linux/Windows are attached to each
[GitHub release](https://github.com/ryansann/okftool/releases).

## Usage

```sh
okftool validate <bundle>      # §9 conformance only (exit 1 if non-conformant)
okftool lint <bundle>          # spec + lint rules; --config, --format pretty|json|sarif
okftool rules                  # list all rules by category
okftool explain <rule>         # a rule's rationale, category, default severity
okftool init [dir]             # scaffold a .okftool.yaml
okftool build <bundle>         # package the bundle as <name>.tar.gz (-o, --prefix)
```

`lint` reads `<bundle>/.okftool.yaml` (or `--config`), else the `okf-recommended`
profile. Exit is non-zero on non-conformance or any diagnostic at/above
`ci.fail-on` (default `error`). `--format sarif` emits SARIF 2.1.0 for inline
GitHub PR annotations.

## Rules & configuration

okftool ships **28 lint rules** across seven categories (frontmatter,
type-vocabulary, linking, topology, graph-structure, body, index/log). `.okftool.yaml` selects a
profile with `extends`, sets per-rule severity/options, scopes rules with glob
`overrides`, and gates CI with `ci.fail-on`. Concepts can opt out inline via an
`okf-lint-disable` frontmatter list.

```yaml
# .okftool.yaml
extends: okf-recommended           # or okf-strict / okf-minimal
rules:
  linking/no-dangling-links: warn          # off | info | warn | error
  topology/max-out-degree: { severity: warn, options: { max: 20 } }
graph:
  neighborhoods:
    graph-authoring:
      paths: [principles/local-neighborhoods.md, rules/graph-coherence.md]
ci:
  fail-on: error
```

The full format, every rule, and the **okf-recommended / okf-strict / okf-minimal**
profiles are documented in okftool's own OKF bundle (it dogfoods itself) — see
[`docs/okf`](docs/okf), in particular
[reference/configuration](docs/okf/reference/configuration.md),
[reference/profiles](docs/okf/reference/profiles.md), and
[reference/rules](docs/okf/reference/rules/index.md).

## Develop

```sh
make ci          # fmt + clippy + test + wasm build + dogfood lint
make lint-self   # lint okftool's own OKF docs bundle under the strict profile
make help        # list all targets
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add a rule.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
