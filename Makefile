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
	$(CARGO) test --workspace --exclude okflint-wasm

.PHONY: wasm
wasm: ## Compile the wasm crate (CI)
	$(CARGO) build -p okflint-wasm --target $(WASM_TARGET) --release

.PHONY: pkg
pkg: ## Build the npm package (bundler target → crates/okflint-wasm/pkg)
	$(WASM_PACK) build crates/okflint-wasm --target bundler --out-name okflint --out-dir pkg

.PHONY: build
build: ## Release build of the CLI
	$(CARGO) build -p okflint-cli --release

.PHONY: install
install: ## Install the okflint CLI from source
	$(CARGO) install --path crates/okflint-cli

.PHONY: ci
ci: fmt-check lint test wasm ## Everything CI runs

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf crates/okflint-wasm/pkg pkg
