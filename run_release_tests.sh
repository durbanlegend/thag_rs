#!/usr/bin/env bash
# Release Quality Check Script for thag_rs v0.2.0
# Runs all tests and checks from RELEASE_CHECKLIST.md

set -e  # Exit on error (but we'll handle errors ourselves)
set -o pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track failures
declare -a FAILURES=()
TOTAL_CHECKS=0
PASSED_CHECKS=0

# Helper functions
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_step() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

run_check() {
    local description="$1"
    shift
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    print_step "$description"
    if "$@"; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        print_success "PASSED: $description"
        return 0
    else
        FAILURES+=("$description")
        print_error "FAILED: $description"
        return 1
    fi
}

# Change to project root
cd "$(dirname "$0")"

echo -e "${GREEN}Starting Release Quality Checks${NC}"
echo "Time started: $(date)"

# ============================================================================
# WORKSPACE TESTS
# ============================================================================
print_header "Workspace Tests"

run_check "Test workspace (default features)" \
    cargo test --workspace

run_check "Test workspace (env_logger variant)" \
    cargo test --workspace --no-default-features --features env_logger,full || true

run_check "Test workspace (with tools)" \
    cargo test --workspace --features tools

run_check "Test workspace (with profiling)" \
    cargo test --workspace --features profiling

print_step "Running integration tests (--test-threads=1)"
run_check "Integration tests" \
    cargo test --test integration_test -- --test-threads=1

# ============================================================================
# SUBCRATE TESTS AND CLIPPY
# ============================================================================
print_header "Subcrate Tests and Clippy"

# thag_common
print_step "Testing thag_common..."
cd thag_common
run_check "thag_common: tests" cargo test
run_check "thag_common: clippy" cargo clippy -- -W clippy::pedantic
cd ..

# thag_proc_macros
print_step "Testing thag_proc_macros..."
cd thag_proc_macros
run_check "thag_proc_macros: tests" cargo test
run_check "thag_proc_macros: clippy" cargo clippy -- -W clippy::pedantic
cd ..

# thag_styling
print_step "Testing thag_styling..."
cd thag_styling
run_check "thag_styling: tests (basic)" cargo test --features basic
run_check "thag_styling: tests (full)" cargo test --features full
run_check "thag_styling: clippy" cargo clippy --features full -- -W clippy::pedantic
cd ..

# thag_profiler
print_step "Testing thag_profiler..."
cd thag_profiler
run_check "thag_profiler: tests (default)" cargo test
run_check "thag_profiler: tests (full_profiling)" cargo test --features full_profiling
run_check "thag_profiler: clippy" cargo clippy --features full_profiling -- -W clippy::pedantic
cd ..

# thag_demo
print_step "Testing thag_demo..."
cd thag_demo
run_check "thag_demo: tests" cargo test
run_check "thag_demo: clippy" cargo clippy -- -W clippy::pedantic
cd ..

# ============================================================================
# MAIN WORKSPACE QUALITY CHECKS
# ============================================================================
print_header "Main Workspace Quality Checks"

run_check "Build release (workspace)" \
    cargo build --release --workspace

run_check "Clippy (workspace default)" \
    cargo clippy --all-targets --workspace

run_check "Clippy (workspace env_logger)" \
    cargo clippy --workspace --no-default-features --features env_logger,core || true

run_check "Format check" \
    cargo fmt --all -- --check

run_check "Documentation build" \
    cargo doc --workspace --no-deps

# ============================================================================
# PROSE QUALITY CHECKS
# ============================================================================
print_header "Prose Quality Checks"

if command -v typos &> /dev/null; then
    run_check "Typos check" typos
else
    print_error "typos not installed - skipping"
    FAILURES+=("typos (not installed)")
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
fi

if command -v vale &> /dev/null; then
    run_check "Vale: README.md" vale README.md --no-wrap || true
    run_check "Vale: thag_profiler/README.md" vale thag_profiler/README.md --no-wrap || true
    run_check "Vale: thag_styling/README.md" vale thag_styling/README.md --no-wrap || true
else
    print_error "vale not installed - skipping vale checks"
    FAILURES+=("vale (not installed)")
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
fi

# ============================================================================
# SUMMARY
# ============================================================================
print_header "Summary"

echo "Time completed: $(date)"
echo ""
echo "Total checks: $TOTAL_CHECKS"
echo "Passed: $PASSED_CHECKS"
echo "Failed: ${#FAILURES[@]}"
echo ""

if [ ${#FAILURES[@]} -eq 0 ]; then
    print_success "ALL CHECKS PASSED! 🎉"
    echo ""
    echo "Next steps:"
    echo "  1. Review version numbers in Cargo.toml files"
    echo "  2. Run cargo msrv verify"
    echo "  3. Run package dry-runs"
    echo "  4. Proceed with release!"
    exit 0
else
    print_error "SOME CHECKS FAILED:"
    echo ""
    for failure in "${FAILURES[@]}"; do
        echo "  - $failure"
    done
    echo ""
    echo "Please fix the failures before proceeding with release."
    exit 1
fi
