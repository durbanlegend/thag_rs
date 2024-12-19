#!/bin/bash

LOG_DIR="dependency_logs"
mkdir -p "$LOG_DIR"

track_crate() {
    local crate_name=$1
    local LOG_FILE="$LOG_DIR/${crate_name}_history.csv"

    # Create log file if doesn't exist
    if [ ! -f "$LOG_FILE" ]; then
        echo "Date,Direct Dependencies,Indirect Dependencies,Total Size (MB)" > "$LOG_FILE"
    fi

    # Get direct dependencies count
    DIRECT_DEPS=$(cargo metadata --format-version=1 --no-deps | \
        jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .dependencies[] | .name')
    DIRECT_COUNT=$(echo "$DIRECT_DEPS" | grep -c "^" || echo "0")

    # Get transitive dependencies for this crate
    ALL_DEPS=$(cargo metadata --format-version=1 | \
        jq -r --arg name "$crate_name" \
        '.resolve.nodes[] | select(.id | startswith($name)) | .deps[] | .name' | \
        sort -u)
    INDIRECT_COUNT=$(comm -23 <(echo "$ALL_DEPS" | sort) <(echo "$DIRECT_DEPS" | sort) | grep -c "^" || echo "0")

    # Calculate size for this crate's dependencies
    TOTAL_SIZE=$(find "../target/debug/deps" -name "lib${crate_name}*.rlib" -exec du -m {} + | awk '{sum += $1} END {print sum}')

    DATE=$(date +%Y-%m-%d)
    echo "$DATE,$DIRECT_COUNT,$INDIRECT_COUNT,$TOTAL_SIZE" >> "$LOG_FILE"

    echo "=== $crate_name Tracking Report ==="
    echo "Date: $DATE"
    echo "Direct Dependencies: $DIRECT_COUNT"
    echo "Indirect Dependencies: $INDIRECT_COUNT"
    echo "Total Size: ${TOTAL_SIZE}MB"
}

# Track each crate
for crate in thag thag_core thag_proc_macros; do
    track_crate "$crate"
    echo -e "\n-------------------\n"
done
