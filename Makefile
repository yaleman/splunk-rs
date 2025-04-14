.DEFAULT: help
.PHONY: help
help:
	@grep -E -h '\s##\s' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt
fmt: ## Run cargo fmt --check
fmt:
	cargo fmt --check


.PHONY: build
build: ## build docs and the code
build: doc
	cargo build

.PHONY: doc
doc: ## build all the docs
doc:
	cargo doc --no-deps
	@echo "Recreating index redirector file..."
	@echo "<meta http-equiv=\"refresh\" content=\"0; url=splunk\">" > target/doc/index.html

.PHONY: doc/open
doc/open: ## build then open the docs
doc/open: doc
	open target/doc/index.html

.PHONY: watch/doc
watch/doc: ## Watch the source dir and build the docs when they change (you need to "cargo install cargo-watch)"
watch/doc:
	cargo watch -w splunk --no-restart -s 'make doc'

.PHONY: watch/build
watch/doc: ## Watch the source dir and build the library when the source changes.
watch/build:
	cargo watch -w splunk --no-restart -s 'make'

.PHONY: precommit
precommit: ## Do the pre-commit-pre-publish things
precommit: fmt
	cargo clippy --all-targets
	cargo test
	cargo test --release
	cargo build
	cargo build --release
	cargo outdated
	cargo audit
