# okftool

A fast, embeddable validator and linter for [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
(Open Knowledge Format) bundles, written in Rust.

One core, three surfaces:

| Surface | Crate | Use |
|---|---|---|
| **Native CLI** | `okftool-cli` → `okftool` | CI, local `okftool validate` / `okftool lint` |
| **wasm / npm** | `okftool-wasm` → `okftool` npm pkg | embed in JS hosts (desktop app, Node/edge API, browser) |
| **Rust crate** | `okftool-core` | embed directly in a Rust host |

The brain — parsing, spec validation, the lint engine — lives in `okftool-core`
and is filesystem-agnostic (callers pass `(path, content)`), so it compiles
unchanged to native and `wasm32`. There is exactly one implementation.

## Two layers

okftool mirrors the OKF spec's MUST/SHOULD split:

- **`validate`** enforces what the spec says MUST be true (§9 conformance:
  parseable frontmatter, non-empty `type`). These are non-disableable errors.
- **`lint`** enforces what SHOULD be true plus everything the spec leaves to
  judgment. Every lint rule is configurable and disableable. A lint rule may
  never flag something the spec mandates tolerating (broken links, unknown
  `type` values, missing optional fields) as a non-disableable error.

## CLI

```sh
okftool validate <bundle>          # §9 conformance only (exit 1 if non-conformant)
okftool lint <bundle>              # spec + lint rules; --config, --format pretty|json|sarif
okftool rules                      # list all rules by category
okftool explain <rule>             # a rule's rationale, category, default severity
okftool init [dir]                 # scaffold a .okftool.yaml
```

`lint` reads `<bundle>/.okftool.yaml` (or `--config`), else the `okf-recommended`
profile. Exit is non-zero on non-conformance or any diagnostic at/above
`ci.fail-on` (default `error`).

### Configuration

`.okftool.yaml` selects a profile with `extends`, sets per-rule severity/options,
scopes rules with glob `overrides`, and gates CI with `ci.fail-on`. Concepts can
opt out inline via an `okf-lint-disable` frontmatter list.

The full format, every rule, and the **okf-recommended / okf-strict / okf-minimal**
profiles are documented in okftool's own OKF bundle — see
[docs/okf](docs/okf) ([reference/configuration](docs/okf/reference/configuration.md),
[reference/profiles](docs/okf/reference/profiles.md)).

## Develop

```sh
make ci                                           # fmt + clippy + test + wasm
cargo run -p okftool-cli -- lint fixtures/cases/all-rules
wasm-pack build crates/okftool-wasm --target bundler --out-name okftool   # npm pkg
```

## Status

Phases 0–3 complete: parser, §9 `validate`, the lint engine (9 rules across 5
categories), `.okftool.yaml` config with presets/overrides/inline-disable, and
the full CLI — all running identically on the native and wasm builds. Next:
autofix (`--fix`) + `okf index`, the rest of the rule catalog, the release
pipeline, and wiring okfview's Diagnostics panel to the wasm package.

## License

Apache-2.0
