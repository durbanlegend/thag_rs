#!/bin/bash

# Script to compare thag_profiler vs dhat-rs memory profiling results
# Usage: ./compare_profilers.sh

echo "=== Profiler Comparison Script ==="
echo

# Clean up any existing results
rm -f dhat-heap.json

echo "1. Running with thag_profiler only..."
echo "----------------------------------------"
THAG_PROFILER=memory,,announce,true thag --features full_profiling demo/thag_profile_benchmark.rs -f
echo

echo "2. Running with dhat-rs profiler..."
echo "----------------------------------------"
thag --features dhat-heap demo/thag_profile_benchmark.rs -f
echo

if [ -f "dhat-heap.json" ]; then
    echo "3. dhat-rs results summary:"
    echo "----------------------------------------"
    # Extract key metrics from dhat JSON
    if command -v jq >/dev/null 2>&1; then
        echo "Overall peak memory usage: $(jq '[.pps[].mb] | max' dhat-heap.json) bytes"
        echo "Overall total allocations: $(jq '[.pps[].tbk] | add' dhat-heap.json) blocks"
        echo "Overall total bytes allocated: $(jq '[.pps[].tb] | add' dhat-heap.json) bytes"
        echo
        echo "Per-function allocation breakdown:"

        # Find ALL frame indices for our functions
        allocate_vectors_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_vectors")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        allocate_hashmap_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_hashmap")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        allocate_deallocate_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_and_deallocate")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        nested_allocations_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("nested_allocations")) | .key | tostring' dhat-heap.json | tr '\n' ' ')

        # Extract stats for each function (accumulating all frame indices)
        if [ ! -z "$allocate_vectors_indices" ] && [ "$allocate_vectors_indices" != " " ]; then
            indices_array=$(echo "$allocate_vectors_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            vectors_bytes=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
            vectors_blocks=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tbk] | add // 0' dhat-heap.json)
            echo "  allocate_vectors: $vectors_bytes bytes, $vectors_blocks blocks (indices: $allocate_vectors_indices)"
        else
            echo "  allocate_vectors: not found in frame table"
        fi

        if [ ! -z "$allocate_hashmap_indices" ] && [ "$allocate_hashmap_indices" != " " ]; then
            indices_array=$(echo "$allocate_hashmap_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            hashmap_bytes=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
            hashmap_blocks=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tbk] | add // 0' dhat-heap.json)
            echo "  allocate_hashmap: $hashmap_bytes bytes, $hashmap_blocks blocks (indices: $allocate_hashmap_indices)"
        else
            echo "  allocate_hashmap: not found in frame table"
        fi

        if [ ! -z "$allocate_deallocate_indices" ] && [ "$allocate_deallocate_indices" != " " ]; then
            indices_array=$(echo "$allocate_deallocate_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            deallocate_bytes=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
            deallocate_blocks=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tbk] | add // 0' dhat-heap.json)
            echo "  allocate_and_deallocate: $deallocate_bytes bytes, $deallocate_blocks blocks (indices: $allocate_deallocate_indices)"
        else
            echo "  allocate_and_deallocate: not found in frame table"
        fi

        if [ ! -z "$nested_allocations_indices" ] && [ "$nested_allocations_indices" != " " ]; then
            indices_array=$(echo "$nested_allocations_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            nested_bytes=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
            nested_blocks=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tbk] | add // 0' dhat-heap.json)
            echo "  nested_allocations: $nested_bytes bytes, $nested_blocks blocks (indices: $nested_allocations_indices)"
        else
            echo "  nested_allocations: not found in frame table"
        fi
    else
        echo "Install 'jq' for detailed JSON parsing, or check dhat-heap.json manually"
        echo "Key fields to check: pps array with tb (total bytes), tbk (total blocks), mb (max bytes), mbk (max blocks)"
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
