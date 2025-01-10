#!/bin/bash
# analyze_single_crate.sh

CRATE_NAME="$1"
if [ -z "$CRATE_NAME" ]; then
    echo "Usage: $0 <crate-name>"
    exit 1
fi

# Function to get size in MB for a dependency
get_size_mb() {
    local dep=$1
    # Extract crate name without version
    local crate_name=$(echo "$dep" | cut -d' ' -f1 | sed 's/-/_/g')
    # Find matching .rlib and sum sizes in KB, then convert to MB
    local size_kb=$(find "../../target/debug/deps" -name "lib${crate_name}-*.rlib" -exec du -k {} + 2>/dev/null | awk '{sum += $1} END {print sum}')
    if [ -z "$size_kb" ]; then
        echo "0.00"
    else
        printf "%.2f" $(echo "scale=2; $size_kb/1024" | bc)
    fi
}

echo "=== Analyzing $CRATE_NAME ==="

# Get direct dependencies with sizes
echo -e "\nDirect Dependencies:"
DIRECT_DEPS=$(cargo tree -p "$CRATE_NAME" -e=no-dev --depth 1 | grep "^[└├]" | sed 's/[└├]── //')
DIRECT_TOTAL=0
while IFS= read -r dep; do
    if [ ! -z "$dep" ]; then
        size=$(get_size_mb "$dep")
        printf "%-60s %6.2f MB\n" "$dep" "$size"
        DIRECT_TOTAL=$(echo "$DIRECT_TOTAL + $size" | bc)
    fi
done <<< "$DIRECT_DEPS"
DIRECT_COUNT=$(echo "$DIRECT_DEPS" | grep -c "^")
echo "Direct dependency count: $DIRECT_COUNT"
printf "Total direct dependencies size: %.2f MB\n" "$DIRECT_TOTAL"

# Get full tree with improved processing
ALL_DEPS=$(cargo tree -p "$CRATE_NAME" -e=no-dev,no-build | \
    egrep -v "^$CRATE_NAME" | \
    egrep -v '^├──' | \
    egrep -v ' \(\*\)' | \
    sed 's/│//g' | \
    sed 's/├//g' | \
    sed 's/└//g' | \
    sed 's/─//g' | \
    sed -E 's/^ +//g' | \
    sort -u)

# Get indirect dependencies with sizes
echo -e "\nIndirect Dependencies:"
INDIRECT_DEPS=$(comm -23 <(echo "$ALL_DEPS" | sort) <(echo "$DIRECT_DEPS" | sort))
INDIRECT_TOTAL=0
while IFS= read -r dep; do
    if [ ! -z "$dep" ]; then
        size=$(get_size_mb "$dep")
        printf "%-60s %6.2f MB\n" "$dep" "$size"
        INDIRECT_TOTAL=$(echo "$INDIRECT_TOTAL + $size" | bc)
    fi
done <<< "$INDIRECT_DEPS"
INDIRECT_COUNT=$(echo "$INDIRECT_DEPS" | grep -c "^" || echo "0")
echo "Indirect dependency count: $INDIRECT_COUNT"
printf "Total indirect dependencies size: %.2f MB\n" "$INDIRECT_TOTAL"

# Calculate and show total size
TOTAL_SIZE=$(echo "$DIRECT_TOTAL + $INDIRECT_TOTAL" | bc)
printf "\nTotal dependencies size: %.2f MB\n" "$TOTAL_SIZE"

# Show feature usage
echo -e "\nFeature Usage:"
cargo metadata --format-version=1 --no-deps | \
    jq -r --arg name "$CRATE_NAME" \
    '.packages[] | select(.name == $name) | .features'
