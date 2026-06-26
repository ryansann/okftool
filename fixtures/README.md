# Test fixtures

Small OKF bundles used by the test suite.

## `cases/` (committed)

One tiny bundle per diagnostic, each crafted to trigger **exactly** one rule (or
validation) and nothing else — they double as worked examples. Asserted by
`crates/okftool-core/tests/cases.rs`.

| Case | Triggers | Notes |
|---|---|---|
| `conformant` | — | clean baseline; zero diagnostics |
| `missing-type` | `missing-type` (spec) | non-conformant |
| `missing-frontmatter` | `missing-frontmatter` (spec) | non-conformant |
| `orphan-concept` | `topology/no-orphan-concepts` | |
| `relative-links` | `linking/prefer-absolute-links` | |
| `type-casing` | `type-vocabulary/consistent-type-casing` | `Widget` vs `widget` |
| `sparse-frontmatter` | `frontmatter/require-description`, `frontmatter/require-timestamp` | |
| `bad-timestamp` | `frontmatter/timestamp-iso8601` | |
| `prose-wall` | `body/structural-body` | |
| `dangling-link` | `linking/no-dangling-links` | needs the rule enabled (off by default) |
| `hub-overflow` | `topology/max-out-degree` | needs a low `max` option |
| `all-rules` | several | kitchen-sink; asserted by `tests/golden.rs` |

## `.vendor/` (gitignored)

The large real-world parity corpus (`ga4`, `stackoverflow`, `crypto_bitcoin`,
vendored from okfview). Used by `tests/fixtures.rs` to prove okftool's verdict
matches the app on real bundles; those tests **skip** when this directory is
absent, so they don't run in CI.
