#!/bin/bash --norc

CRATE_NAME="$1"
if [ -z "$CRATE_NAME" ]; then
    echo "Usage: $0 <crate-name>"
    exit 1
fi

# Function to safely output formatted text
safe_printf() {
    local format="$1"
    shift
    printf "$format\n" "$@" 2>/dev/null || true
}

# Function to list unique build timestamps
get_build_timestamps() {
    local latest=0  # Initialize to 0
    # Get the most recent timestamp
    for file in ../../target/debug/deps/*.rlib; do
        if [ -f "$file" ]; then
            local timestamp=$(stat -f "%m" "$file" 2>/dev/null || echo "0")
            if [ "$timestamp" -gt 0 ] && [ "$timestamp" -gt "$latest" ]; then
                latest=$timestamp
            fi
        fi
    done
    echo "$latest"
}

# Function to add decimal numbers safely
add_numbers() {
    local num1="$1"
    local num2="$2"
    echo "scale=2; $num1 + $num2" | bc | sed 's/^\./0./'
}

# Function to get size in MB for a dependency
get_size_mb() {
    local dep="$1"
    local crate_name=$(echo "$dep" | cut -d' ' -f1 | sed 's/-/_/g')

    local size_kb=0
    for lib in ../../target/debug/deps/lib${crate_name}-*.rlib; do
        if [ -f "$lib" ]; then
            local this_size=$(du -k "$lib" 2>/dev/null | cut -f1)
            size_kb=$((size_kb + ${this_size:-0}))
        fi
    done

    if [ $size_kb -eq 0 ]; then
        echo "0.00"
    else
        echo "scale=2; $size_kb/1024" | bc | sed 's/^\./0./'
    fi
}

# Analyze dependencies and sort by size
analyze_deps() {
    local deps="$1"
    local total=0.00
    local count=0
    local temp_file=$(mktemp)

    while IFS= read -r dep; do
        if [ -n "$dep" ]; then
            local size=$(get_size_mb "$dep")
            echo "$size $dep" >> "$temp_file"
            total=$(add_numbers "$total" "$size")
            ((count++))
        fi
    done <<< "$deps"

    # Sort by size and display
    if [ -f "$temp_file" ]; then
        sort -rn "$temp_file" | while read -r size dep; do
            safe_printf "%-60s %6.2f MB" "$dep" "$size"
        done
        rm "$temp_file"
    fi

    echo "Count: $count"
    safe_printf "Total size: %.2f MB" "$total"

    echo "$total"
}

echo "=== Analyzing $CRATE_NAME ==="

# Get most recent build timestamp
BUILD_TIME=$(get_build_timestamps)
if [ -n "$BUILD_TIME" ]; then
    echo "Using build from: $(date -r "$BUILD_TIME")"
else
    echo "No builds found"
    exit 1
fi

# Direct dependencies analysis
echo -e "\nDirect Dependencies (sorted by size):"
DIRECT_DEPS=$(cargo tree -p "$CRATE_NAME" -e=no-dev --depth 1 | grep "^[└├]" | sed 's/[└├]── //')
DIRECT_TOTAL=$(analyze_deps "$DIRECT_DEPS")

# Get full dependency tree
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

# Indirect dependencies analysis
echo -e "\nIndirect Dependencies (sorted by size):"
INDIRECT_DEPS=$(comm -23 <(echo "$ALL_DEPS" | sort) <(echo "$DIRECT_DEPS" | sort))
INDIRECT_TOTAL=$(analyze_deps "$INDIRECT_DEPS")

# Summary statistics
echo -e "\nDependency Summary:"
safe_printf "Direct dependencies size:    %6.2f MB\n" "$DIRECT_TOTAL"
safe_printf "Indirect dependencies size:  %6.2f MB\n" "$INDIRECT_TOTAL"
TOTAL_SIZE=$(add_numbers "$DIRECT_TOTAL" "$INDIRECT_TOTAL")
safe_printf "Total size:                 %6.2f MB\n" "$TOTAL_SIZE"

# Feature usage
echo -e "\nFeature Usage:"
cargo metadata --format-version=1 --no-deps | \
    jq -r --arg name "$CRATE_NAME" \
    '.packages[] | select(.name == $name) | .features'

# Dependency chain analysis
echo -e "\nLongest Dependency Chains:"
cargo tree -p "$CRATE_NAME" -e=no-dev --prefix=depth | \
    grep -v '(*)' | \
    awk '{ print length($0) " " $0 }' | \
    sort -rn | \
    head -n 5 | \
    cut -d' ' -f2-
