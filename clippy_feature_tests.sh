#!/bin/bash

# Array of checks to run
CHECKS=(
    "check-ast"
    "check-build"
    "check-build-alt"
    "check-core"
    "check-core-alt"
    "check-detect"
    "check-full"
    "check-repl"
    "check-tui"
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
