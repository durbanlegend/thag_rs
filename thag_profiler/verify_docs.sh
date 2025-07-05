#!/bin/bash

# Script to verify that the internal-docs feature correctly shows/hides documentation

set -e

echo "Verifying thag_profiler documentation feature..."
echo

# Clean previous builds
cargo clean --package thag_profiler

# Build public API docs
echo "1. Building public API documentation..."
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps --quiet

# Check that internal functions are hidden in public docs
PUBLIC_DOC_DIR="target/doc/thag_profiler"
if [ -f "$PUBLIC_DOC_DIR/index.html" ]; then
    echo "✓ Public documentation generated"

    # Check for hidden items (these should NOT appear in public docs)
    HIDDEN_ITEMS=0

    # Check for thousands function (should be hidden)
    if grep -q "thousands" "$PUBLIC_DOC_DIR/fn.thousands.html" 2>/dev/null; then
        echo "✗ ERROR: 'thousands' function found in public docs (should be hidden)"
        HIDDEN_ITEMS=$((HIDDEN_ITEMS + 1))
    else
        echo "✓ 'thousands' function properly hidden in public docs"
    fi

    # Check for init_profiling function (should be hidden)
    if grep -q "init_profiling" "$PUBLIC_DOC_DIR/fn.init_profiling.html" 2>/dev/null; then
        echo "✗ ERROR: 'init_profiling' function found in public docs (should be hidden)"
        HIDDEN_ITEMS=$((HIDDEN_ITEMS + 1))
    else
        echo "✓ 'init_profiling' function properly hidden in public docs"
    fi

    # Check for __private module (should be hidden)
    if grep -q "__private" "$PUBLIC_DOC_DIR/index.html" 2>/dev/null; then
        echo "✗ ERROR: '__private' module found in public docs (should be hidden)"
        HIDDEN_ITEMS=$((HIDDEN_ITEMS + 1))
    else
        echo "✓ '__private' module properly hidden in public docs"
    fi

    echo "   Public docs hidden items check: $HIDDEN_ITEMS errors"
else
    echo "✗ Public documentation not found"
    exit 1
fi

echo

# Build internal docs
echo "2. Building internal documentation..."
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal-docs --no-deps --quiet

# Check that internal functions are visible in internal docs
INTERNAL_DOC_DIR="target/doc/thag_profiler"
if [ -f "$INTERNAL_DOC_DIR/index.html" ]; then
    echo "✓ Internal documentation generated"

    # Check for visible items (these SHOULD appear in internal docs)
    VISIBLE_ITEMS=0

    # Check for thousands function (should be visible)
    if [ -f "$INTERNAL_DOC_DIR/fn.thousands.html" ]; then
        echo "✓ 'thousands' function visible in internal docs"
    else
        echo "✗ ERROR: 'thousands' function not found in internal docs (should be visible)"
        VISIBLE_ITEMS=$((VISIBLE_ITEMS + 1))
    fi

    # Check for init_profiling function (should be visible)
    if [ -f "$INTERNAL_DOC_DIR/fn.init_profiling.html" ]; then
        echo "✓ 'init_profiling' function visible in internal docs"
    else
        echo "✗ ERROR: 'init_profiling' function not found in internal docs (should be visible)"
        VISIBLE_ITEMS=$((VISIBLE_ITEMS + 1))
    fi

    # Check for __private module (should be visible)
    if grep -q "__private" "$INTERNAL_DOC_DIR/index.html" 2>/dev/null; then
        echo "✓ '__private' module visible in internal docs"
    else
        echo "✗ ERROR: '__private' module not found in internal docs (should be visible)"
        VISIBLE_ITEMS=$((VISIBLE_ITEMS + 1))
    fi

    echo "   Internal docs visibility check: $VISIBLE_ITEMS errors"
else
    echo "✗ Internal documentation not found"
    exit 1
fi

echo

# Summary
TOTAL_ERRORS=$((HIDDEN_ITEMS + VISIBLE_ITEMS))
if [ $TOTAL_ERRORS -eq 0 ]; then
    echo "✅ All documentation feature tests passed!"
    echo "   - Public API docs properly hide internal details"
    echo "   - Internal docs properly show implementation details"
else
    echo "❌ Documentation feature tests failed with $TOTAL_ERRORS errors"
    echo "   - Check that #[cfg_attr(not(feature = \"internal-docs\"), doc(hidden))] is applied correctly"
    exit 1
fi

echo
echo "Documentation feature verification complete."
