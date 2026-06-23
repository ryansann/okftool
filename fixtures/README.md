# Test fixtures

Small OKF bundles used by the test suite.

## `cases/` (committed)

One tiny bundle per diagnostic, each crafted to trigger **exactly** one rule (or
validation) and nothing else — they double as worked examples. Asserted by
`crates/okflint-core/tests/cases.rs`.

| Case | Triggers | Notes |
|---|---|---|
| `conformant` | — | clean baseline; zero diagnostics |
| `missing-type` | `missing-type` (spec) | non-conformant |
| `missing-frontmatter` | `missing-frontmatter` (spec) | non-conformant |
| `orphan-concept` | `no-orphan-concepts` | |
| `relative-links` | `prefer-absolute-links` | |
| `type-casing` | `consistent-type-casing` | `Widget` vs `widget` |
| `sparse-frontmatter` | `require-description`, `require-timestamp` | |
| `bad-timestamp` | `timestamp-iso8601` | |
| `prose-wall` | `structural-body` | |
| `dangling-link` | `no-dangling-links` | needs the rule enabled (off by default) |
| `hub-overflow` | `max-out-degree` | needs a low `max` option |
| `all-rules` | several | kitchen-sink; asserted by `tests/golden.rs` |

## `.vendor/` (gitignored)

The large real-world parity corpus (`ga4`, `stackoverflow`, `crypto_bitcoin`,
vendored from okfview). Used by `tests/fixtures.rs` to prove okflint's verdict
matches the app on real bundles; those tests **skip** when this directory is
absent, so they don't run in CI.
