# Security Policy

## Supported versions

okftool is pre-1.0; security fixes land on the latest released minor version.

## Reporting a vulnerability

Please report security issues **privately** via GitHub's
[private vulnerability reporting](https://github.com/ryansann/okftool/security/advisories/new)
(Security → Advisories → "Report a vulnerability"). Do not open a public issue
for a suspected vulnerability.

We aim to acknowledge a report within a few days and will coordinate a fix and
disclosure timeline with you.

## Scope notes

okftool reads untrusted Markdown/YAML and produces diagnostics; it does not
execute bundle content. The CLI reads files from a path you provide. The wasm
build performs no IO — the host passes content in. Reports about parsing
crashes, panics on malformed input, or resource exhaustion are in scope.
