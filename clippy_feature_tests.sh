#!/bin/bash

# Array of checks to run
CHECKS=(
    "check-core"
    "check-build"
    "check-ast"
    "check-detect"
    "check-color-old"
    "check-full"
    "check-tui"
    "check-core-alt"
    "check-build-alt"
)

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Running clippy checks across feature combinations..."

for check in "${CHECKS[@]}"; do
    echo -e "\n${GREEN}Running: cargo ${check}${NC}"
    if ! cargo ${check}; then
        echo -e "${RED}Clippy failed for check: ${check}${NC}"
        exit 1
    fi
done

echo -e "\n${GREEN}All clippy checks passed!${NC}"
