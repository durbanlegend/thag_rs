#!/bin/bash

#!/bin/bash

analyze_crate() {
    local crate_name=$1
    local output=""

    output+="=== Analyzing $crate_name ===\n\n"
    output+="Direct Dependencies:\n"

    # Get and count direct dependencies
    DIRECT_DEPS=$(cargo metadata --format-version=1 --no-deps | \
        jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .dependencies[] | .name' | \
        sort)
    DIRECT_COUNT=$(echo "$DIRECT_DEPS" | grep -c "^")

    output+="$DIRECT_DEPS\n"
    output+="Count: $DIRECT_COUNT\n\n"

    output+="Indirect Dependencies:\n"

    # Get all transitive dependencies for this crate
    ALL_DEPS=$(cargo metadata --format-version=1 | \
        jq -r --arg name "$crate_name" \
        '.resolve.nodes[] | select(.id | startswith($name)) | .deps[] | .name' | \
        sort -u)

    # Filter out direct dependencies to get indirect ones
    INDIRECT_DEPS=$(comm -23 <(echo "$ALL_DEPS" | sort) <(echo "$DIRECT_DEPS" | sort))
    INDIRECT_COUNT=$(echo "$INDIRECT_DEPS" | grep -c "^" || echo "0")

    output+="$INDIRECT_DEPS\n"
    output+="Count: $INDIRECT_COUNT\n"

    # Print everything at once
    echo -e "$output"
}

# Analyze each crate
for crate in thag thag_core thag_proc_macros; do
# for crate in thag_proc_macros; do
    analyze_crate "$crate"
    echo -e "\n-------------------\n"
done
