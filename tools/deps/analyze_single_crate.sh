#!/bin/bash
# analyze_single_crate.sh

CRATE_NAME="$1"
if [ -z "$CRATE_NAME" ]; then
    echo "Usage: $0 <crate-name>"
    exit 1
fi

echo "=== Analyzing $CRATE_NAME ==="

# Get direct dependencies
echo -e "\nDirect Dependencies:"
DIRECT_DEPS=$(cargo tree -p "$CRATE_NAME" -e=no-dev --depth 1 | grep "^[└├]" | sed 's/[└├]── //')
echo "$DIRECT_DEPS"
DIRECT_COUNT=$(echo "$DIRECT_DEPS" | grep -c "^")
echo "Direct dependency count: $DIRECT_COUNT"

# Get full tree with your improved processing
echo -e "\nFull Dependency Tree:"
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
echo "$ALL_DEPS"

# Get indirect dependencies by removing direct deps from all deps
echo -e "\nIndirect Dependencies:"
INDIRECT_DEPS=$(comm -23 <(echo "$ALL_DEPS" | sort) <(echo "$DIRECT_DEPS" | sort))
echo "$INDIRECT_DEPS"
INDIRECT_COUNT=$(echo "$INDIRECT_DEPS" | grep -c "^" || echo "0")
echo "Indirect dependency count: $INDIRECT_COUNT"

# Show feature usage
echo -e "\nFeature Usage:"
cargo metadata --format-version=1 --no-deps | \
    jq -r --arg name "$CRATE_NAME" \
    '.packages[] | select(.name == $name) | .features'
