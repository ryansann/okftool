# okftool docs

## `okf/` — okftool, documented as an OKF bundle

okftool's own documentation is written as an [OKF](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
bundle and lives at [`okf/`](./okf). It's a deliberate dogfood: the tool documents
itself in the format it validates, with one `LintRule` concept per rule (the
human-readable counterpart to `okftool explain <rule>`).

Because it's a real OKF bundle, okftool checks it like any other:

```sh
okftool validate docs/okf     # spec conformance
okftool lint     docs/okf     # spec + lint rules (expected: clean)
```

CI runs both, so the docs can't drift out of conformance.

Layout:

```
okf/
  index.md                 root table of contents (okf_version: "0.1")
  architecture/            overview + one concept per crate
  reference/
    validation.md          the spec (§9) layer
    configuration.md       the .okftool.yaml format
    profiles.md            the okf-recommended / strict / minimal presets
    rules/                 one LintRule concept per lint rule
```
