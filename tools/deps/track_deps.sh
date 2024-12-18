#!/bin/bash

LOG_DIR="dependency_logs"
mkdir -p "$LOG_DIR"

track_crate() {
    local crate_name=$1
    local LOG_FILE="$LOG_DIR/${crate_name}_history.csv"

    # Create log file if doesn't exist
    if [ ! -f "$LOG_FILE" ]; then
        echo "Date,Direct Dependencies,Indirect Dependencies,Total Size" > "$LOG_FILE"
    fi

    # Get counts
    DIRECT_COUNT=$(cargo metadata --format-version=1 --no-deps | \
        jq -r --arg name "$crate_name" '.packages[] | select(.name == $name) | .dependencies[] | .name' | \
        wc -l)

    ALL_DEPS_COUNT=$(cargo metadata --format-version=1 | \
        jq -r '.resolve.nodes[] | select(.id | contains("thag") | not) | .id' | \
        cut -d' ' -f1 | sort -u | wc -l)

    INDIRECT_COUNT=$((ALL_DEPS_COUNT - DIRECT_COUNT))

    # Get size
    TOTAL_SIZE=$(find "../../target/debug/deps" -name "*.rlib" -exec du -m -c {} + | tail -n1 | cut -f1)

    # Log data
    DATE=$(date +%Y-%m-%d)
    echo "$DATE,$DIRECT_COUNT,$INDIRECT_COUNT,$TOTAL_SIZE" >> "$LOG_FILE"

    echo "=== $crate_name Tracking Report ==="
    echo "Date: $DATE"
    echo "Direct Dependencies: $DIRECT_COUNT"
    echo "Indirect Dependencies: $INDIRECT_COUNT"
    echo "Total Size: $TOTAL_SIZE MB"
}

# Track each crate
for crate in thag thag_core thag_proc_macros; do
    track_crate "$crate"
    echo -e "\n-------------------\n"
done
