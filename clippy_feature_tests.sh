#!/bin/bash

# Array of feature combinations to test
FEATURE_SETS=(
    "--lib --no-default-features --features=core,simplelog"
    "--no-default-features --features=build,simplelog"
    "--features=ast"
    "--features=color_detect"
    "--features=color_support"
    "--features=full"
    "--features=tui"
    "--lib --no-default-features --features=core,env_logger"
    "--no-default-features --features=build,env_logger"
)

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "Running clippy checks across feature combinations..."

for features in "${FEATURE_SETS[@]}"; do
    echo -e "\n${GREEN}Testing with: cargo clippy ${features}${NC}"
    if ! cargo clippy ${features} -- -D warnings -W clippy::pedantic; then
        echo -e "${RED}Clippy failed for feature set: ${features}${NC}"
        exit 1
    fi
done

echo -e "\n${GREEN}All clippy checks passed!${NC}"
