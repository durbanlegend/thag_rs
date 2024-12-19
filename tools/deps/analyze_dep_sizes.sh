#!/bin/bash

echo "=== Dependency Sizes ==="

# Ensure we're in a package directory, not workspace root
if [ -n "$1" ]; then
    cd "$1" || exit 1
fi

# Build first to ensure dependencies exist
cargo build

# Function to convert bytes to human-readable format
human_size() {
    local kb=$1
    if [ $kb -gt 1024 ]; then
        echo "$(echo "scale=2; $kb/1024" | bc)MB"
    else
        echo "${kb}KB"
    fi
}

# Get and process sizes
echo -e "\nIndividual Dependency Sizes:"
cargo metadata --format-version=1 | \
    jq -r '.resolve.nodes[] | select(.id | contains("thag") | not) | .id' | \
    while read -r dep; do
        # Extract just the crate name without version
        dep_name=$(echo "$dep" | sed -E 's/.*#([^@]+)@.*/\1/')
        size=$(find "../target/debug/deps" -name "lib${dep_name}*.rlib" 2>/dev/null | \
               xargs du -k 2>/dev/null | cut -f1)
        if [ ! -z "$size" ]; then
            printf "%-40s %15s\n" "$dep_name" "$(human_size $size)"
        fi
    done | sort -h -k2 -r

echo -e "\nTotal Size:"
total_size=$(find "../target/debug/deps" -name "*.rlib" -exec du -k {} + | awk '{sum += $1} END {print sum}')
echo "$(human_size $total_size)"
