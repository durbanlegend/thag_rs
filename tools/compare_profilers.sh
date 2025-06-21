#!/bin/bash

# Script to compare thag_profiler vs dhat-rs memory profiling results
# Usage: ./compare_profilers.sh

echo "=== Profiler Comparison Script ==="
echo

# Clean up any existing results
rm -f dhat-heap.json

echo "1. Running with thag_profiler only..."
echo "----------------------------------------"
THAG_PROFILER=memory thag --features full_profiling demo/thag_profile_benchmark.rs -f

# Find the most recent thag_profiler folded file
folded_file=$(ls -t thag_profile_benchmark-*-memory.folded 2>/dev/null | head -1)
if [ -f "$folded_file" ]; then
    echo
    echo "thag_profiler results from: $folded_file"
    echo "----------------------------------------"

    # Extract per-function allocations from folded format
    echo "Per-function allocation breakdown (thag_profiler):"

    # Parse the folded file for our target functions
    vectors_bytes=$(grep "allocate_vectors" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    hashmap_bytes=$(grep "allocate_hashmap" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    deallocate_bytes=$(grep "allocate_and_deallocate" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    nested_bytes=$(grep "nested_allocations" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')

    echo "  allocate_vectors: $vectors_bytes bytes"
    echo "  allocate_hashmap: $hashmap_bytes bytes"
    echo "  allocate_and_deallocate: $deallocate_bytes bytes"
    echo "  nested_allocations: $nested_bytes bytes"
    echo
    echo "For graph run 'thag_profile .'"
    echo "For detailed profiling, first rerun with:"
    echo "THAG_PROFILER=memory,,announce,true thag --features full_profiling demo/thag_profile_benchmark.rs -ft"
else
    echo "No thag_profiler folded file found"
fi
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
echo "4. Side-by-side comparison:"
echo "----------------------------------------"
if [ -f "$folded_file" ]; then
    echo "Function                    | thag_profiler | dhat-rs      | Difference"
    echo "----------------------------|---------------|--------------|------------"

    # Get thag_profiler values
    thag_vectors=$(grep "allocate_vectors" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    thag_hashmap=$(grep "allocate_hashmap" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    thag_deallocate=$(grep "allocate_and_deallocate" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')
    thag_nested=$(grep "nested_allocations" "$folded_file" | grep -v " 0$" | awk '{sum += $NF} END {print sum+0}')

    # Get dhat values (if available)
    if [ -f "dhat-heap.json" ] && command -v jq >/dev/null 2>&1; then
        allocate_vectors_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_vectors")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        allocate_hashmap_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_hashmap")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        allocate_deallocate_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("allocate_and_deallocate")) | .key | tostring' dhat-heap.json | tr '\n' ' ')
        nested_allocations_indices=$(jq -r '.ftbl | to_entries[] | select(.value | contains("nested_allocations")) | .key | tostring' dhat-heap.json | tr '\n' ' ')

        if [ ! -z "$allocate_vectors_indices" ] && [ "$allocate_vectors_indices" != " " ]; then
            indices_array=$(echo "$allocate_vectors_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            dhat_vectors=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
        else
            dhat_vectors=0
        fi

        if [ ! -z "$allocate_hashmap_indices" ] && [ "$allocate_hashmap_indices" != " " ]; then
            indices_array=$(echo "$allocate_hashmap_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            dhat_hashmap=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
        else
            dhat_hashmap=0
        fi

        if [ ! -z "$allocate_deallocate_indices" ] && [ "$allocate_deallocate_indices" != " " ]; then
            indices_array=$(echo "$allocate_deallocate_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            dhat_deallocate=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
        else
            dhat_deallocate=0
        fi

        if [ ! -z "$nested_allocations_indices" ] && [ "$nested_allocations_indices" != " " ]; then
            indices_array=$(echo "$nested_allocations_indices" | tr ' ' '\n' | grep -v '^$' | jq -R 'tonumber' | jq -s '.')
            dhat_nested=$(jq --argjson indices "$indices_array" '[.pps[] | select(.fs as $fs | $indices | any(. as $idx | $fs | contains([$idx]))) | .tb] | add // 0' dhat-heap.json)
        else
            dhat_nested=0
        fi

        # Calculate differences
        diff_vectors=$((dhat_vectors - thag_vectors))
        diff_hashmap=$((dhat_hashmap - thag_hashmap))
        diff_deallocate=$((dhat_deallocate - thag_deallocate))
        diff_nested=$((dhat_nested - thag_nested))

        printf "%-27s | %13s | %12s | %+d\n" "allocate_vectors" "$thag_vectors" "$dhat_vectors" "$diff_vectors"
        printf "%-27s | %13s | %12s | %+d\n" "allocate_hashmap" "$thag_hashmap" "$dhat_hashmap" "$diff_hashmap"
        printf "%-27s | %13s | %12s | %+d\n" "allocate_and_deallocate" "$thag_deallocate" "$dhat_deallocate" "$diff_deallocate"
        printf "%-27s | %13s | %12s | %+d\n" "nested_allocations" "$thag_nested" "$dhat_nested" "$diff_nested"
    else
        echo "dhat data not available for comparison"
    fi
else
    echo "thag_profiler data not available for comparison"
fi

echo
echo "5. Analysis notes:"
echo "----------------------------------------"
echo "- Compare allocation totals between both profilers"
echo "- Check for significant discrepancies (>10% difference)"
echo "- Verify allocation patterns match expected behavior"
echo
echo "Expected patterns:"
echo "- allocate_vectors: ~1MB (1000 vectors * 1024 bytes)"
echo "- allocate_hashmap: ~400-500KB (500 HashMap entries + overhead)"
echo "- allocate_and_deallocate: ~24MB (temporary allocations)"
echo "- nested_allocations: Variable (nested allocation pattern)"
