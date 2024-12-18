#!/bin/bash

analyze_crate() {
    local crate_name=$1
    echo "=== Analyzing $crate_name ==="

    echo -e "\nDirect Dependencies:"
    cargo metadata --format-version=1 --no-deps | \
        jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .dependencies[] | .name' | \
        sort | \
        tee >(wc -l | xargs -I{} echo "Count: {}")

    echo -e "\nIndirect Dependencies:"
    cargo metadata --format-version=1 | \
        jq -r '.resolve.nodes[] | select(.id | contains("thag") | not) | .id' | \
        cut -d' ' -f1 | \
        sort | \
        grep -vF -f <(cargo metadata --format-version=1 --no-deps | \
            jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .dependencies[] | .name') | \
        tee >(wc -l | xargs -I{} echo "Count: {}")
}

# Analyze each crate
#for crate in thag thag_core thag_proc_macros; do
for crate in thag_proc_macros; do
    analyze_crate "$crate"
    echo -e "\n-------------------\n"
done
