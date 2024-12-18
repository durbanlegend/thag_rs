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
    local bytes=$1
    if [ $bytes -gt 1048576 ]; then
        echo "$(echo "scale=2; $bytes/1048576" | bc)MB"
    elif [ $bytes -gt 1024 ]; then
        echo "$(echo "scale=2; $bytes/1024" | bc)KB"
    else
        echo "${bytes}B"
    fi
}

# Get and process sizes
echo -e "\nIndividual Dependency Sizes:"
cargo metadata --format-version=1 | \
    jq -r '.resolve.nodes[] | select(.id | contains("thag") | not) | .id' | \
    while read -r dep; do
        dep_name="${dep%% *}"
        # echo "dep=$dep, dep_name=$dep_name"
        size=$(find "../target/debug/deps" -name "lib${dep_name}*.rlib" 2>/dev/null | \
               xargs du -m 2>/dev/null | cut -f1)
        if [ ! -z "$size" ]; then
            printf "%-40s %15s\n" "$dep_name" "$(human_size $size)"
        fi
    done | sort -h -k2 -r

# Show total size
echo -e "\nTotal Size:"
pwd
total_size=$(find "../target/debug/deps" -name "*.rlib" -exec du -mc {} + | tail -n1 | cut -f1)
echo "$(human_size $total_size)"
