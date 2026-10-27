CORE := core
DEFAULT_FEATURES := core,ely,fabric,neoforge
FFI_FEATURES := ffi
MKDOCS ?= venv/bin/mkdocs

.PHONY: all test test-ffi test-core test-ely test-fabric test-neoforge \
	    fmt fmt-check clippy clippy-ffi clippy-core lint check \
	    build build-ffi build-lib dist \
	    docs docs-serve docs-check \
	    clean help

all: test

test:
	cargo test

test-ffi:
	cargo test --features "$(FFI_FEATURES)"

test-core:
	cargo test --no-default-features --features "$(CORE)"

test-ely:
	cargo test --no-default-features --features ely

test-fabric:
	cargo test --no-default-features --features fabric

test-neoforge:
	cargo test --no-default-features --features neoforge

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

clippy-ffi:
	cargo clippy --all-targets --features "$(FFI_FEATURES)" -- -D warnings

clippy-core:
	cargo clippy --all-targets --no-default-features --features "$(CORE)" -- -D warnings

lint: fmt-check clippy clippy-ffi

check: test test-ffi lint docs-check

build:
	cargo build --release

build-ffi:
	cargo build --release --features "$(FFI_FEATURES)"

build-lib:
	cargo build --release --no-default-features --features "$(CORE)"

dist: build-ffi
	mkdir -p dist
	@for f in target/release/*.so target/release/*.dylib target/release/*.dll target/release/*.a target/release/*.lib; do \
	    if [ -e "$$f" ]; then cp "$$f" dist/; fi; \
	done
	@echo "Artifacts in dist/:"
	@ls -la dist/

docs:
	$(MKDOCS) build

docs-serve:
	$(MKDOCS) serve

docs-check:
	$(MKDOCS) build --strict

clean:
	cargo clean
	rm -rf dist site

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  test          Run tests (default features: $(DEFAULT_FEATURES))"
	@echo "  test-ffi      Run tests with ffi feature (needed for tests/ffi.rs)"
	@echo "  test-core     Run tests with only 'core' (minimal build)"
	@echo "  test-ely      Run tests with only 'ely'"
	@echo "  test-fabric   Run tests with only 'fabric'"
	@echo "  test-neoforge Run tests with only 'neoforge'"
	@echo ""
	@echo "  fmt           Format code"
	@echo "  fmt-check     Check formatting"
	@echo "  clippy        Clippy (default features, -D warnings)"
	@echo "  clippy-ffi    Clippy with ffi (-D warnings)"
	@echo "  clippy-core   Clippy with only 'core' (-D warnings)"
	@echo "  lint          fmt-check + clippy + clippy-ffi"
	@echo "  check         test + test-ffi + lint + docs-check"
	@echo ""
	@echo "  build         cargo build --release (rlib + cdylib, no ffi)"
	@echo "  build-ffi     cargo build --release --features ffi ->"
	@echo "                Linux: librmlib.so  macOS: librmlib.dylib  Windows: rmlib.dll"
	@echo "  build-lib     cargo build --release with only 'core' (minimal rlib)"
	@echo "  dist          build-ffi, then copy artifacts into dist/"
	@echo ""
	@echo "  docs          Build the MkDocs site into site/"
	@echo "  docs-serve    Serve the docs with live reload"
	@echo "  docs-check    Build the docs in strict mode (warnings become errors)"
	@echo ""
	@echo "  clean         cargo clean + rm -rf dist site"
	@echo "  help          Show this help"
	@echo ""
	@echo "Note: 'MKDOCS' defaults to 'venv/bin/mkdocs'. Override with: make docs MKDOCS=mkdocs"
