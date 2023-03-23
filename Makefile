.DEFAULT: build

.PHONY: fmt
fmt:
	cargo fmt --check


.PHONY: build
build: doc
	cargo build

.PHONY: doc
doc:
	cargo doc --no-deps
	@echo "Recreating index redirector file..."
	@echo "<meta http-equiv=\"refresh\" content=\"0; url=splunk\">" > target/doc/index.html

.PHONY: doc/open
doc/open: doc
	open target/doc/index.html

.PHONY: watch/doc
watch/doc:
	cargo watch -w splunk --no-restart -s 'make doc'

.PHONY: watch/build
watch/build:
	cargo watch -w splunk --no-restart -s 'make'

