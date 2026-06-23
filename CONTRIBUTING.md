# Contributing to okftool

Thanks for your interest in improving okftool! This guide covers the workflow and
the one thing worth knowing well: how to add a lint rule.

## Getting started

You need a recent Rust toolchain (see `rust-toolchain.toml`) and, for the wasm
build, [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

```sh
make ci          # fmt + clippy (warnings denied) + tests + wasm build + dogfood lint
make test        # tests only
make help        # all targets
```

Every change must keep `make ci` green: `cargo fmt`, `clippy -D warnings`, the
test suite, and the dogfood (okftool linting its own `docs/okf` bundle under the
strict profile) all pass.

## Architecture

One core, three thin surfaces:

- `crates/okftool-core` — the parser, the `validate` (§9 conformance) layer, and
  the lint engine. **No IO** — callers pass `(path, content)` — so it compiles
  unchanged to native and `wasm32`.
- `crates/okftool-cli` — the `okftool` binary (filesystem + formatting only).
- `crates/okftool-wasm` — `wasm-bindgen` bindings for the npm package.

Keep all logic in `okftool-core`; the CLI and wasm crates should stay thin.

## Adding a lint rule

A rule is a small struct in `crates/okftool-core/src/lint/rules.rs`. The pattern:

1. **Implement `Rule`** — a `meta()` (id, category, summary, rationale, default
   severity, fixable) and a `check(ctx) -> Vec<Finding>`. The default severity is
   the rule's recommended-profile default; the catalog's hard rule is that a lint
   rule may never be a non-disableable error for something the spec tolerates.
2. **Register it** in `registry()`.
3. **Add a fixture** under `fixtures/cases/<rule>/` that triggers *only* that rule
   (plus clean partners), and a `Case` entry in
   `crates/okftool-core/tests/cases.rs` asserting the exact code set.
4. **Document it** with a `LintRule` concept in
   `docs/okf/reference/rules/<id>.md` and add it to that directory's `index.md`.
5. **Update the affected presets** (`presets/okf-strict.yaml`,
   `presets/okf-minimal.yaml`) and re-sync the embedded copies in
   `docs/okf/reference/profiles.md` (a test guards this).
6. If your rule fires on the `fixtures/cases/all-rules` kitchen-sink, update the
   expected set in `tests/golden.rs`. If it would fire on the self-doc bundle,
   fix the docs so they stay clean (the dogfood must pass under strict).

Run `make ci` and you'll be told if any of the above is out of sync.

## Pull requests

- Keep PRs focused; one logical change per PR.
- Include tests. New rules need a fixture case and a doc concept.
- Describe the *why*, not just the *what*.

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License, Version 2.0](LICENSE).
