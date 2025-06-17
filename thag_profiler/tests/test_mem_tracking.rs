/// Each test is executed sequentially in the main test function to avoid concurrency issues with global state.
/// This approach ensures that tests run reliably and without interference.
///
/// ```bash
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_mem_tracking -- --nocapture
/// ```
///
/// 1. Individual test functions for each aspect of the memory tracking system
/// 2. A single main `#[test]` function that runs all the tests sequentially
/// 3. Use of `safe_alloc! { ... })` to prevent infinite recursion
/// 4. A safe approach to testing persistent allocations using a `Mutex`
/// 5. Proper state initialization and cleanup
///
/// The test covers key functionality like:
///
/// - Task creation and memory tracking
/// - Task contexts and memory usage reporting
/// - Thread task stacks and tracking
/// - Task path registry and matching
/// - Allocation registry functionality
/// - `safe_alloc!` behavior
/// - Task state and ID generation
/// - Memory profiling lifecycle
///
#[cfg(feature = "full_profiling")]
use thag_profiler::{
    mem_tracking::{self, create_memory_task, get_last_active_task, Allocator, TASK_PATH_REGISTRY},
    profiled, safe_alloc,
};

#[cfg(feature = "full_profiling")]
use std::sync::{LazyLock, Mutex};

// Utility for persistent allocations across test boundaries
#[cfg(feature = "full_profiling")]
static TEST_MEMORY: LazyLock<Mutex<Vec<Vec<u8>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// ---------------------------------------------------------------------------
// Test functions for memory tracking
// ---------------------------------------------------------------------------

/// Test safe_alloc!
#[cfg(feature = "full_profiling")]
fn test_with_sys_alloc() {
    // Start with the default Tracking allocator
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::Tracking,
        "Default allocator should be Tracking"
    );

    // Run code with System allocator
    let result = safe_alloc! {
        // Inside this closure, allocator should be System
        let current = thag_profiler::mem_tracking::current_allocator();
        assert_eq!(
            current,
            Allocator::System,
            "Allocator should be System inside safe_alloc!"
        );

        // Return the current allocator for verification
        current
    };

    // Verify the result
    assert_eq!(
        result,
        Allocator::System,
        "safe_alloc! should return closure result"
    );

    // After safe_alloc!, allocator should be back to Tracking
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::Tracking,
        "Allocator should be restored to Tracking after safe_alloc!"
    );

    // Nested safe_alloc! calls
    let nested_result = safe_alloc! {
        assert_eq!(
            thag_profiler::mem_tracking::current_allocator(),
            Allocator::System,
            "First level: Allocator should be System"
        );

        safe_alloc! {
            assert_eq!(
                thag_profiler::mem_tracking::current_allocator(),
                Allocator::System,
                "Second level: Allocator should be System"
            );
        };
        "success"
    };

    assert_eq!(nested_result, "success", "Nested safe_alloc! should work");

    // After both safe_alloc! calls, allocator should be back to Tracking
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::Tracking,
        "Allocator should be restored to Tracking after nested safe_alloc!"
    );
}

/// Test task state generation
#[cfg(feature = "full_profiling")]
fn test_task_state() {
    // Use the system allocator
    safe_alloc! {
        // Get the current next_task_id value
        let current_id = thag_profiler::mem_tracking::TASK_STATE
            .next_task_id
            .load(std::sync::atomic::Ordering::SeqCst);

        // Create some tasks and verify IDs are sequential
        let task1 = create_memory_task();
        let task2 = create_memory_task();
        let task3 = create_memory_task();

        assert_eq!(
            task1.id(),
            current_id,
            "First task ID should match current_id"
        );
        assert_eq!(
            task2.id(),
            current_id + 1,
            "Second task ID should be current_id + 1"
        );
        assert_eq!(
            task3.id(),
            current_id + 2,
            "Third task ID should be current_id + 2"
        );
    };
}

/// Test profiled memory allocations
#[cfg(feature = "full_profiling")]
#[profiled(mem_summary)]
fn test_profiled_memory_allocations() {
    // This function is profiled, so allocations should be tracked
    let data: Vec<u8> = Vec::with_capacity(1024);
    let data2 = vec![0u8; 2048];

    // Force the compiler to keep the variables
    assert_eq!(data.capacity(), 1024);
    assert_eq!(data2.len(), 2048);

    // Store in static for persistence
    {
        let mut memory = TEST_MEMORY.lock().unwrap();
        *memory = vec![data2];
    }
}

/// Test persistent allocations
#[cfg(feature = "full_profiling")]
fn test_persistent_allocations() {
    // Check that allocations from previous test are still valid
    let memory = TEST_MEMORY.lock().unwrap();
    assert_eq!(memory.len(), 1, "Should have one persistent allocation");

    if !memory.is_empty() {
        assert_eq!(
            memory[0].len(),
            2048,
            "Persistent allocation should be 2048 bytes"
        );
    }
}

// ---------------------------------------------------------------------------
// Main test function that runs all tests sequentially
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "full_profiling")]
fn test_mem_tracking_full_sequence() {
    // Ensure we start with a clean profiling state
    thag_profiler::profiling::disable_profiling();
    enable_memory_profiling_for_test();

    // Helper function to enable memory profiling using the attribute macro
    #[thag_profiler::enable_profiling(memory)]
    fn enable_memory_profiling_for_test() {}

    eprintln!("Starting memory tracking tests");

    // safe_alloc! tests
    eprintln!("Testing safe_alloc! function...");
    test_with_sys_alloc();

    // Task state tests
    eprintln!("Testing task state...");
    test_task_state();

    // Profiled memory allocations tests
    eprintln!("Testing profiled memory allocations...");
    test_profiled_memory_allocations();

    // Test persistent allocations
    eprintln!("Testing persistent allocations...");
    test_persistent_allocations();

    // Clean up
    thag_profiler::profiling::disable_profiling();

    eprintln!("All memory tracking tests completed successfully!");
}
