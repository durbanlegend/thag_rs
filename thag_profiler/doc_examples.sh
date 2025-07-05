#!/bin/bash

# Documentation build examples for thag_profiler
# This script demonstrates how to build documentation with different visibility levels

set -e

echo "Building thag_profiler documentation..."
echo

# Clean previous builds
cargo clean --package thag_profiler

echo "1. Building PUBLIC API documentation (clean, user-facing):"
echo "   cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps"
echo
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps

echo
echo "2. Building INTERNAL documentation (includes implementation details and private items):"
echo "   cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --document-private-items"
echo
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --document-private-items

echo
echo "Documentation built successfully!"
echo
echo "To view the documentation:"
echo "  Public API:  cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps --open"
echo "  Internal:    cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --document-private-items --open"
echo
echo "The key difference:"
echo "  - Without 'internal-docs' feature: Clean API focused on end users"
echo "  - With 'internal-docs' feature: Includes implementation details, internal utilities, debugging tools, and private functions"
