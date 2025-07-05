# Makefile for thag_profiler documentation and common tasks

.PHONY: doc doc-internal doc-public doc-open doc-internal-open clean test lint fmt help

# Default target
help:
	@echo "Available targets:"
	@echo "  doc-public      - Build public API documentation (clean, user-facing)"
	@echo "  doc-internal    - Build internal documentation (includes implementation details and private items)"
	@echo "  doc-open        - Build and open public API documentation"
	@echo "  doc-internal-open - Build and open internal documentation (with private items)"
	@echo "  doc             - Build both public and internal documentation"
	@echo "  clean           - Clean build artifacts"
	@echo "  test            - Run tests"
	@echo "  lint            - Run clippy linter"
	@echo "  fmt             - Format code"
	@echo "  help            - Show this help message"

# Documentation targets
doc-public:
	@echo "Building public API documentation (clean, user-facing)..."
	cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps

doc-internal:
	@echo "Building internal documentation (includes implementation details and private items)..."
	cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --document-private-items

doc-open:
	@echo "Building and opening public API documentation..."
	cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps --open

doc-internal-open:
	@echo "Building and opening internal documentation..."
	cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --document-private-items --open

doc: doc-public doc-internal
	@echo "Both documentation modes built successfully!"

# Development targets
clean:
	cargo clean --package thag_profiler

test:
	cargo test --package thag_profiler

lint:
	cargo clippy --package thag_profiler --all-targets --all-features

fmt:
	cargo fmt --package thag_profiler

# Convenience aliases
docs: doc
doc-pub: doc-public
doc-int: doc-internal
