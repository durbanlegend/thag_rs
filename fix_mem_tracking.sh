#!/bin/bash
# Script to fix the thread panic in mem_tracking.rs

# Path to the mem_tracking.rs file
MEM_TRACKING_FILE="thag_rs/thag_profiler/src/mem_tracking.rs"

# Make a backup
cp "$MEM_TRACKING_FILE" "${MEM_TRACKING_FILE}.bak"

# Fix the initialize_thread_allocator_context function
# This changes:
# 1. Safely access thread-local storage with panic protection
# 2. Use try_borrow_mut instead of borrow_mut to avoid panics
# 3. Add early return if already borrowed

TMP_FILE=$(mktemp)

# Process the file
cat "$MEM_TRACKING_FILE" | sed -E '
# Find the initialize_thread_allocator_context function and modify it
/pub fn initialize_thread_allocator_context\(\)/ {
  # Print the function definition
  p
  # Read the next line assuming it contains the CURRENT_ALLOCATOR_CONTEXT.with call
  n
  # Replace it with our safer version
  c\
    // Get the inherited allocator context safely\
    let inherited = match std::panic::catch_unwind(|| {\
        CURRENT_ALLOCATOR_CONTEXT.with(|cell| cell.get())\
    }) {\
        Ok(alloc) => alloc,\
        Err(_) => {\
            // Default to system allocator if thread local access fails\
            Allocator::System\
        }\
    };\
\
    ALLOCATOR_STATE.with(|state| {
  # Read the next line which should have state.borrow_mut()
  n
  # Replace it with try_borrow_mut
  c\
        // Try to borrow the state mutably, but don\'t panic if already borrowed\
        let state_result = state.try_borrow_mut();\
        if state_result.is_err() {\
            return; // Skip if already borrowed\
        }\
        let mut state = state_result.unwrap();
}' > "$TMP_FILE"

# Apply the changes
mv "$TMP_FILE" "$MEM_TRACKING_FILE"

echo "Applied fix to $MEM_TRACKING_FILE"
echo "Original file backed up to ${MEM_TRACKING_FILE}.bak"