#!/bin/bash

# Script to compare thag_profiler vs dhat-rs memory profiling results
# Usage: ./compare_profilers.sh

echo "=== Profiler Comparison Script ==="
echo

# Clean up any existing results
rm -f dhat-heap.json

echo "1. Running with thag_profiler only..."
echo "----------------------------------------"
thag --features full_profiling tools/thag_profile_benchmark.rs -f
echo

echo "2. Running with dhat-rs profiler..."
echo "----------------------------------------"
thag --features dhat-heap tools/thag_profile_benchmark.rs -f
echo

if [ -f "dhat-heap.json" ]; then
    echo "3. dhat-rs results summary:"
    echo "----------------------------------------"
    # Extract key metrics from dhat JSON
    if command -v jq >/dev/null 2>&1; then
        echo "Peak memory usage: $(jq '.dhatFileVersion.peak_blocks' dhat-heap.json) blocks"
        echo "Total allocations: $(jq '.dhatFileVersion.total_blocks' dhat-heap.json) blocks"
        echo "Total bytes allocated: $(jq '.dhatFileVersion.total_bytes' dhat-heap.json) bytes"
    else
        echo "Install 'jq' for detailed JSON parsing, or check dhat-heap.json manually"
        echo "Key fields to check: peak_blocks, total_blocks, total_bytes"
    fi
    echo
    echo "Full dhat results available in: dhat-heap.json"
    echo "View with: https://nnethercote.github.io/dh_view/dh_view.html"
else
    echo "No dhat-heap.json found - dhat may not have run correctly"
fi

echo
echo "4. Comparison notes:"
echo "----------------------------------------"
echo "- Compare peak memory usage between both profilers"
echo "- Check total allocation counts"
echo "- Verify allocation/deallocation patterns"
echo "- Look for any significant discrepancies"
echo
echo "Expected patterns:"
echo "- Test 1: ~1MB peak (1000 vectors * 1024 bytes)"
echo "- Test 2: ~400KB (500 HashMap entries * ~800 bytes)"
echo "- Test 3: Temporary spikes during loop iterations"
echo "- Test 4: Complex nested allocation pattern"
