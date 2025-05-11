#!/bin/bash

# Run the simplified chat server script
echo "Running simplified chat server..."
cd "$(dirname "$0")"
cargo run --bin thag -- run demo/smol_chat_server_profile_simple.rs

# If the server has completed, display the location of the profiling data
echo ""
echo "Checking for most recent profiling files..."
find $TMPDIR/thag_profiler -type f -name "*-memory.folded" -o -name "*-debug.log" | sort | tail -n 5
