/// Each test is executed sequentially in the main test function to avoid concurrency issues with global state.
/// This approach ensures that tests run reliably and without interference.
///
/// ```bash
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_mem_tracking -- --nocapture
/// ```
///
/// 1. Individual test functions for each aspect of the memory tracking system
/// 2. A single main `#[test]` function that runs all the tests sequentially
/// 3. Use of `with_allocator(Allocator::System, || { ... })` to prevent infinite recursion
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
/// - `with_allocator` behavior
/// - Task state and ID generation
/// - Memory profiling lifecycle
///
#[cfg(feature = "full_profiling")]
use thag_profiler::{
    enable_profiling,
    mem_tracking::{
        activate_task, create_memory_task, get_active_tasks, get_last_active_task,
        get_task_memory_usage, /*, pop_task_from_stack, push_task_to_stack */
        record_alloc_for_task_id, with_allocator, Allocator, TaskGuard, ALLOC_REGISTRY,
        TASK_PATH_REGISTRY,
    },
    profiled,
    profiling::ProfileType,
};

#[cfg(feature = "full_profiling")]
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

// Utility for persistent allocations across test boundaries
#[cfg(feature = "full_profiling")]
static TEST_MEMORY: LazyLock<Mutex<Vec<Vec<u8>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// ---------------------------------------------------------------------------
// Test functions for memory tracking
// ---------------------------------------------------------------------------

/// Test basic allocation tracking with task IDs
#[cfg(feature = "full_profiling")]
fn test_memory_task_allocation() {
    // Use the system allocator to avoid recursive tracking
    with_allocator(Allocator::System, || {
        // Create a memory task
        let memory_task = create_memory_task();
        let task_id = memory_task.id();

        eprintln!("Created memory task with ID: {}", task_id);

        // Check that the task ID is valid
        assert!(task_id > 0, "Task ID should be greater than zero");

        // Activate the task
        activate_task(task_id);

        // Verify task activation
        let active_tasks = get_active_tasks();
        assert!(
            active_tasks.contains(&task_id),
            "Task should be in active tasks list"
        );

        // Test allocations
        {
            // Record some allocations
            record_alloc_for_task_id(0x1000, 1024, task_id);
            record_alloc_for_task_id(0x2000, 2048, task_id);

            // Check memory usage
            let usage = get_task_memory_usage(task_id);
            assert_eq!(
                usage,
                Some(3072),
                "Memory usage should be sum of allocations"
            );

            // Check memory usage via context
            assert_eq!(
                memory_task.memory_usage(),
                Some(3072),
                "Memory context should report correct usage"
            );
        }

        // Create a guard and verify it removes the task on drop
        {
            let _guard = TaskGuard::new(task_id);
            assert!(
                get_active_tasks().contains(&task_id),
                "Task should be active while guard exists"
            );

            // Let guard go out of scope
        }

        // Verify task was deactivated
        assert!(
            !get_active_tasks().contains(&task_id),
            "Task should be deactivated after guard drops"
        );
    });
}

/// Test task context
#[cfg(feature = "full_profiling")]
fn test_memory_task_context() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
        // Create a memory task
        let memory_task = create_memory_task();
        let task_id = memory_task.id();

        // Task should already be activated
        assert!(
            get_active_tasks().contains(&task_id),
            "Task should be active"
        );

        // Create a guard for the task_id.
        let guard = TaskGuard::new(task_id);

        // Record an allocation
        record_alloc_for_task_id(0x3000, 4096, task_id);

        // Verify memory usage
        assert_eq!(
            memory_task.memory_usage(),
            Some(4096),
            "Memory context should report allocation"
        );

        // Exit context by dropping the guard
        drop(guard);

        // Task should be deactivated
        assert!(
            !get_active_tasks().contains(&task_id),
            "Task should be deactivated after guard drops"
        );
    });
}

// /// Test thread task stacks
// #[cfg(feature = "full_profiling")]
// fn test_thread_task_stacks() {
//     // Use the system allocator
//     with_allocator(Allocator::System, || {
//         // Create multiple tasks
//         let task1 = create_memory_task();
//         let task2 = create_memory_task();
//         let task3 = create_memory_task();

//         let task1_id = task1.id();
//         let task2_id = task2.id();
//         let task3_id = task3.id();

//         let thread_id = thread::current().id();

//         // Push tasks onto stack in different order
//         push_task_to_stack(thread_id, task1_id);
//         push_task_to_stack(thread_id, task2_id);
//         push_task_to_stack(thread_id, task3_id);

//         // Last active task should be the most recently pushed
//         assert_eq!(
//             get_last_active_task(),
//             Some(task3_id),
//             "Last active task should be the most recently pushed"
//         );

//         // Pop middle task
//         pop_task_from_stack(thread_id, task2_id);

//         // Last active task should still be task3
//         assert_eq!(
//             get_last_active_task(),
//             Some(task3_id),
//             "Last active task should still be task3 after popping task2"
//         );

//         // Active tasks should not include task2
//         let active_tasks = get_active_tasks();
//         assert!(active_tasks.contains(&task1_id), "Task1 should be active");
//         assert!(active_tasks.contains(&task3_id), "Task3 should be active");
//         assert!(
//             !active_tasks.contains(&task2_id),
//             "Task2 should not be active"
//         );

//         // Pop the remaining tasks
//         pop_task_from_stack(thread_id, task1_id);
//         pop_task_from_stack(thread_id, task3_id);

//         // No tasks should be active
//         assert!(
//             get_active_tasks().is_empty(),
//             "No tasks should be active after popping all"
//         );
//     });
// }

/// Test task path registry
#[cfg(feature = "full_profiling")]
fn test_task_path_registry() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
        // Create a task
        let task = create_memory_task();
        let task_id = task.id();

        // Create a path for this task
        let path = vec![
            "module".to_string(),
            "submodule".to_string(),
            "function".to_string(),
        ];

        // Register the path
        {
            let mut registry = TASK_PATH_REGISTRY.lock();
            registry.insert(task_id, path.clone());
        }

        // Test finding matching task ID
        let path_copy = path.clone();
        let matching_id = thag_profiler::mem_tracking::find_matching_task_id(&path_copy);
        assert_eq!(
            matching_id, task_id,
            "Should find the correct task ID for the path"
        );

        // Test with a subset of the path
        let subset_path = vec!["module".to_string(), "submodule".to_string()];
        let partial_match = thag_profiler::mem_tracking::find_matching_task_id(&subset_path);
        assert_eq!(
            partial_match, task_id,
            "Should find task with partial path match"
        );

        // Test with completely different path
        let different_path = vec!["other".to_string(), "path".to_string()];
        let no_match = thag_profiler::mem_tracking::find_matching_task_id(&different_path);

        // With no good match, should return the most recently activated task
        assert_eq!(
            no_match,
            get_last_active_task().unwrap_or(0),
            "Should return last active task when no match found"
        );

        // Clean up
        {
            let mut registry = TASK_PATH_REGISTRY.lock();
            registry.remove(&task_id);
        }
    });
}

/// Test allocation registry
#[cfg(feature = "full_profiling")]
fn test_allocation_registry() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
        // Record some allocations for a task
        let task_id = 1000; // Use arbitrary task ID for testing

        // Clear existing registry first
        {
            let mut registry = ALLOC_REGISTRY.lock();
            *registry = thag_profiler::mem_tracking::AllocationRegistry {
                task_allocations: HashMap::new(),
                task_deallocations: HashMap::new(),
                address_to_task: HashMap::new(),
            };
        }

        // Record allocations
        record_alloc_for_task_id(0x1000, 1024, task_id);
        record_alloc_for_task_id(0x2000, 2048, task_id);
        record_alloc_for_task_id(0x3000, 4096, task_id);

        // Check memory usage
        let usage = get_task_memory_usage(task_id);
        assert_eq!(
            usage,
            Some(7168),
            "Memory usage should be sum of allocations"
        );

        // Check address to task mapping
        {
            let registry = ALLOC_REGISTRY.lock();

            // Verify address mappings
            assert_eq!(
                *registry.address_to_task.get(&0x1000).unwrap(),
                task_id,
                "Address 0x1000 should map to task_id"
            );
            assert_eq!(
                *registry.address_to_task.get(&0x2000).unwrap(),
                task_id,
                "Address 0x2000 should map to task_id"
            );
            assert_eq!(
                *registry.address_to_task.get(&0x3000).unwrap(),
                task_id,
                "Address 0x3000 should map to task_id"
            );

            // Verify allocations
            let allocations = registry.task_allocations.get(&task_id).unwrap();
            assert_eq!(allocations.len(), 3, "Should have 3 allocations for task");

            // Check each allocation
            let mut found_1024 = false;
            let mut found_2048 = false;
            let mut found_4096 = false;

            for (_, size) in allocations {
                match size {
                    1024 => found_1024 = true,
                    2048 => found_2048 = true,
                    4096 => found_4096 = true,
                    _ => {}
                }
            }

            assert!(found_1024, "Should have 1024-byte allocation");
            assert!(found_2048, "Should have 2048-byte allocation");
            assert!(found_4096, "Should have 4096-byte allocation");
        }
    });
}

/// Test with_allocator function
#[cfg(feature = "full_profiling")]
fn test_with_allocator() {
    // Start with the default TaskAware allocator
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::TaskAware,
        "Default allocator should be TaskAware"
    );

    // Run code with System allocator
    let result = with_allocator(Allocator::System, || {
        // Inside this closure, allocator should be System
        let current = thag_profiler::mem_tracking::current_allocator();
        assert_eq!(
            current,
            Allocator::System,
            "Allocator should be System inside with_allocator"
        );

        // Return the current allocator for verification
        current
    });

    // Verify the result
    assert_eq!(
        result,
        Allocator::System,
        "with_allocator should return closure result"
    );

    // After with_allocator, allocator should be back to TaskAware
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::TaskAware,
        "Allocator should be restored to TaskAware after with_allocator"
    );

    // Nested with_allocator calls
    let nested_result = with_allocator(Allocator::System, || {
        assert_eq!(
            thag_profiler::mem_tracking::current_allocator(),
            Allocator::System,
            "First level: Allocator should be System"
        );

        with_allocator(Allocator::TaskAware, || {
            assert_eq!(
                thag_profiler::mem_tracking::current_allocator(),
                Allocator::TaskAware,
                "Second level: Allocator should be TaskAware"
            );
        });

        // After inner with_allocator, should be back to System
        assert_eq!(
            thag_profiler::mem_tracking::current_allocator(),
            Allocator::System,
            "After inner with_allocator, should be back to System"
        );

        "success"
    });

    assert_eq!(
        nested_result, "success",
        "Nested with_allocator should work"
    );

    // After both with_allocator calls, allocator should be back to TaskAware
    assert_eq!(
        thag_profiler::mem_tracking::current_allocator(),
        Allocator::TaskAware,
        "Allocator should be restored to TaskAware after nested with_allocator"
    );
}

/// Test task state generation
#[cfg(feature = "full_profiling")]
fn test_task_state() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
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
    });
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

/// Test trimming backtraces
#[cfg(feature = "full_profiling")]
fn test_trim_backtrace() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
        // Create a backtrace
        let backtrace = backtrace::Backtrace::new();

        // Trim it
        let start_pattern = &*thag_profiler::mem_tracking::ALLOC_START_PATTERN;
        let trimmed = thag_profiler::mem_tracking::trim_backtrace(start_pattern, &backtrace);

        // We can't make strong assertions about the content, but we can check some properties
        eprintln!("Trimmed backtrace has {} entries", trimmed.len());

        // Trimmed backtrace should not contain any "__rust_begin_short_backtrace" entries
        assert!(
            trimmed
                .iter()
                .all(|frame| !frame.contains("__rust_begin_short_backtrace")),
            "Trimmed backtrace should not contain __rust_begin_short_backtrace"
        );
    });
}

/// Test high-level memory profiling functions
#[cfg(feature = "full_profiling")]
fn test_memory_profiling_lifecycle() {
    // Use the system allocator
    with_allocator(Allocator::System, || {
        // Initialize memory profiling
        thag_profiler::mem_tracking::initialize_memory_profiling();

        // Create a task for tracking
        let task = create_memory_task();
        let task_id = task.id();

        // Record some allocations
        record_alloc_for_task_id(0x4000, 8192, task_id);

        // Create a path for this task
        let path = vec!["test_module".to_string(), "test_function".to_string()];

        // Register the path
        {
            let mut registry = TASK_PATH_REGISTRY.lock();
            registry.insert(task_id, path);
        }

        // Finalize memory profiling
        thag_profiler::mem_tracking::finalize_memory_profiling();

        // No specific assertions here as finalize_memory_profiling writes to disk
        // and we don't want to make the test dependent on filesystem details
    });
}

// ---------------------------------------------------------------------------
// Main test function that runs all tests sequentially
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "full_profiling")]
fn test_mem_tracking_full_sequence() {
    // Ensure we start with a clean profiling state
    thag_profiler::profiling::disable_profiling();
    enable_profiling(true, Some(ProfileType::Memory)).expect("Failed to enable profiling");

    eprintln!("Starting memory tracking tests");

    // Basic task allocation tests
    eprintln!("Testing basic task allocation...");
    test_memory_task_allocation();

    // Task context tests
    eprintln!("Testing memory task context...");
    test_memory_task_context();

    // Task path registry tests
    eprintln!("Testing task path registry...");
    test_task_path_registry();

    // Allocation registry tests
    eprintln!("Testing allocation registry...");
    test_allocation_registry();

    // with_allocator tests
    eprintln!("Testing with_allocator function...");
    test_with_allocator();

    // Task state tests
    eprintln!("Testing task state...");
    test_task_state();

    // Profiled memory allocations tests
    eprintln!("Testing profiled memory allocations...");
    test_profiled_memory_allocations();

    // Test persistent allocations
    eprintln!("Testing persistent allocations...");
    test_persistent_allocations();

    // Trim backtrace tests
    eprintln!("Testing trim_backtrace...");
    test_trim_backtrace();

    // Memory profiling lifecycle tests
    eprintln!("Testing memory profiling lifecycle...");
    test_memory_profiling_lifecycle();

    // Clean up
    thag_profiler::profiling::disable_profiling();

    eprintln!("All memory tracking tests completed successfully!");
}
