#!/usr/bin/env bash
set -euo pipefail

WASM_PACK="${WASM_PACK:-wasm-pack}"
PKG_DIR="crates/okftool-wasm/pkg"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR"

"$WASM_PACK" build crates/okftool-wasm \
  --target bundler \
  --out-name okftool \
  --out-dir pkg/browser

"$WASM_PACK" build crates/okftool-wasm \
  --target nodejs \
  --out-name okftool \
  --out-dir pkg/node

mv "$PKG_DIR/node/okftool.js" "$PKG_DIR/node/okftool.cjs"
rm -f "$PKG_DIR/browser/.gitignore" "$PKG_DIR/node/.gitignore"
cp README.md LICENSE "$PKG_DIR/"

node <<'NODE'
const fs = require("fs");
const path = require("path");

const pkgDir = path.join("crates", "okftool-wasm", "pkg");
const browserPkg = JSON.parse(
  fs.readFileSync(path.join(pkgDir, "browser", "package.json"), "utf8")
);

const pkg = {
  name: "@ryansann/okftool",
  version: browserPkg.version,
  type: "module",
  description: browserPkg.description,
  license: browserPkg.license,
  repository: {
    type: "git",
    url: "git+https://github.com/ryansann/okftool.git",
  },
  homepage: browserPkg.homepage,
  main: "./node/okftool.cjs",
  module: "./browser/okftool.js",
  types: "./browser/okftool.d.ts",
  exports: {
    ".": {
      browser: "./browser/okftool.js",
      node: "./node/okftool.cjs",
      import: "./browser/okftool.js",
      default: "./browser/okftool.js",
    },
    "./node": "./node/okftool.cjs",
    "./browser": "./browser/okftool.js",
  },
  files: ["browser", "node", "README.md", "LICENSE"],
  sideEffects: ["./browser/okftool.js", "./browser/snippets/*"],
};

fs.writeFileSync(path.join(pkgDir, "package.json"), `${JSON.stringify(pkg, null, 2)}\n`);
NODE
