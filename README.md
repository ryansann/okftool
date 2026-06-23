# okflint

A fast, embeddable validator and linter for [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
(Open Knowledge Format) bundles, written in Rust.

One core, three surfaces:

| Surface | Crate | Use |
|---|---|---|
| **Native CLI** | `okflint-cli` → `okflint` | CI, local `okflint validate` / `okflint lint` |
| **wasm / npm** | `okflint-wasm` → `okflint` npm pkg | embed in JS hosts (desktop app, Node/edge API, browser) |
| **Rust crate** | `okflint-core` | embed directly in a Rust host |

The brain — parsing, spec validation, the lint engine — lives in `okflint-core`
and is filesystem-agnostic (callers pass `(path, content)`), so it compiles
unchanged to native and `wasm32`. There is exactly one implementation.

## Two layers

okflint mirrors the OKF spec's MUST/SHOULD split:

- **`validate`** enforces what the spec says MUST be true (§9 conformance:
  parseable frontmatter, non-empty `type`). These are non-disableable errors.
- **`lint`** enforces what SHOULD be true plus everything the spec leaves to
  judgment. Every lint rule is configurable and disableable. A lint rule may
  never flag something the spec mandates tolerating (broken links, unknown
  `type` values, missing optional fields) as a non-disableable error.

## Develop

```sh
cargo test -p okflint-core -p okflint-cli          # unit + integration tests
cargo run -p okflint-cli -- validate fixtures/minimal
cargo build -p okflint-wasm --target wasm32-unknown-unknown   # wasm builds
wasm-pack build crates/okflint-wasm --target bundler --out-name okflint   # npm pkg
```

## Status

Phase 0 — workspace skeleton: parse + §9 `validate` on both the native and wasm
builds. Lint engine, config (`.okflint.yaml`), presets, autofix, and releases
are landing in subsequent phases.

## License

Apache-2.0
