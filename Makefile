CARGO     ?= cargo
WASM_PACK ?= wasm-pack
WASM_TARGET = wasm32-unknown-unknown

.DEFAULT_GOAL := help

.PHONY: help
help: ## List available targets
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt
fmt: ## Format all crates
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Check formatting (CI)
	$(CARGO) fmt --all --check

.PHONY: lint
lint: ## Clippy, warnings denied (CI)
	$(CARGO) clippy --workspace --all-targets -- -D warnings

.PHONY: test
test: ## Run the test suite (native crates)
	$(CARGO) test --workspace --exclude okftool-wasm

.PHONY: wasm
wasm: ## Compile the wasm crate (CI)
	$(CARGO) build -p okftool-wasm --target $(WASM_TARGET) --release

.PHONY: pkg
pkg: ## Build the npm package (browser + node targets → crates/okftool-wasm/pkg)
	WASM_PACK=$(WASM_PACK) scripts/build-npm-package.sh

.PHONY: build
build: ## Release build of the CLI
	$(CARGO) build -p okftool-cli --release

.PHONY: lint-self
lint-self: ## Lint okftool's own OKF docs bundle (uses docs/okf/.okftool.yaml → strict)
	$(CARGO) run -q -p okftool-cli -- lint docs/okf

.PHONY: install
install: ## Install the okftool CLI from source
	$(CARGO) install --path crates/okftool-cli

.PHONY: ci
ci: fmt-check lint test wasm lint-self ## Everything CI runs

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf crates/okftool-wasm/pkg pkg
