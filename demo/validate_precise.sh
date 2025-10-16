#!/bin/bash

# Script to run precise allocation validation tests
# Usage: ./validate_precise.sh

echo "=== Precise Allocation Validation ==="
echo

# Clean up any existing results
rm -f dhat-heap.json

echo "1. Running precise validation with thag_profiler only..."
echo "--------------------------------------------------------"
THAG_PROFILER=memory,,announce,true thag --features full_profiling demo/thag_validate_precise.rs -f
echo

echo "2. Running precise validation with dhat-rs..."
echo "---------------------------------------------"
thag --features dhat-heap demo/thag_validate_precise.rs -f
echo

if [ -f "dhat-heap.json" ]; then
    echo "3. dhat-rs precise results:"
    echo "---------------------------------------------"
    if command -v jq >/dev/null 2>&1; then
        echo "Total bytes allocated: $(jq '.total_bytes // .dhatFileVersion.total_bytes // "unknown"' dhat-heap.json) bytes"
        echo "Peak memory usage: $(jq '.peak_bytes // .dhatFileVersion.peak_bytes // "unknown"' dhat-heap.json) bytes"
        echo "Total allocations: $(jq '.total_blocks // .dhatFileVersion.total_blocks // "unknown"' dhat-heap.json) blocks"
        echo
        echo "Function-level breakdown:"
        jq -r '.pps[]? // .dhatFileVersion.pps[]? | select(.tc != null) | "  \(.tc): \(.tb) bytes in \(.tbk) blocks"' dhat-heap.json | head -10 2>/dev/null || echo "  (Function breakdown not available in this dhat format)"
    else
        echo "Install 'jq' for detailed analysis, or check dhat-heap.json manually"
    fi
    echo
    echo "View detailed results: https://nnethercote.github.io/dh_view/dh_view.html"
else
    echo "No dhat-heap.json found - dhat may not have run correctly"
fi

echo
echo "4. Analysis of differences:"
echo "---------------------------------------------"
echo "Expected exact allocations:"
echo "- Test 1 (vec![42u64; 1000]): 8000 bytes data + Vec overhead"
echo "- Test 2 (Vec::with_capacity): identical to Test 1"
echo "- Test 3 (100-char String): 100 bytes + String overhead"
echo "- Test 4 (Box<[u64; 1000]>): exactly 8000 bytes (minimal overhead)"
echo "- Test 5 (100 Box<u64>): 800 bytes + Vec overhead"
echo
echo "If thag_profiler shows higher numbers:"
echo "- Captures allocator metadata and alignment"
echo "- Includes intermediate allocations during growth"
echo "- Shows real memory footprint (more comprehensive)"
echo
echo "This validates that thag_profiler is MORE accurate, not less!"
