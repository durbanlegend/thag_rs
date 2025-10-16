#!/bin/bash

# Run the fixed chat server script
echo "Running original chat server with fixes..."
cd "$(dirname "$0")"
cargo run --bin thag_rs -- run demo/smol_chat_server_profile.rs

# If the server has completed, display the location of the profiling data
echo ""
echo "Checking for most recent profiling files..."
find /tmp/thag_profiler -type f -name "*-memory.folded" -o -name "*-debug.log" | sort | tail -n 5

# Also print status of the program
echo ""
echo "Run 'nc localhost 6000' to connect to the server as a client"
echo "The server should automatically stop after 60 seconds"