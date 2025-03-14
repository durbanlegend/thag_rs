// use serial_test::file_serial;
// use serial_test::is_locked_serially;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::{panic, thread, time::Duration};
use thag_profiler::profiling::{
    dump_profiled_functions, enable_profiling, is_profiled_function, is_profiling_enabled,
    register_profiled_function,
};
use thag_profiler::{profile, ProfileType};

// Static mutex for test synchronization
static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct TestGuard;

impl Drop for TestGuard {
    fn drop(&mut self) {
        let _ = enable_profiling(false, ProfileType::Time);
    }
}

// Helper function to get a mutex guard, even if poisoned
fn get_test_lock() -> MutexGuard<'static, ()> {
    match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poison_error) => {
            // If poisoned, recover the guard anyway
            println!("Warning: Mutex was poisoned from a previous test panic. Recovering...");
            poison_error.into_inner()
        }
    }
}

// Use this before each test
fn setup_test() -> MutexGuard<'static, ()> {
    let guard = get_test_lock();

    // Reset profiling state completely
    let _ = enable_profiling(false, ProfileType::Time);

    // Reset any other global state here
    // ...

    guard
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() + panic::UnwindSafe,
{
    // Acquire the lock for this test
    let _guard = TEST_MUTEX.lock().unwrap();

    // Register the crate under a name that the #[profiled] macro will recognize
    // This simulates what would happen if this was using an imported thag_profiler
    register_profiled_function("test_function", "test_description".to_string());

    // Explicitly disable profiling first to ensure clean state
    let _ = enable_profiling(false, ProfileType::Time);

    // Then enable profiling
    let _ = enable_profiling(true, ProfileType::Time);

    // Verify profiling is actually enabled
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled at the start of run_test"
    );

    // Create guard that will clean up even if test panics
    let _guard = TestGuard;

    // Run the test, catching any panics to ensure our guard runs
    let result = panic::catch_unwind(test);

    // Re-throw any panic after our guard has cleaned up
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// Basic profiling tests

#[test]
// #[file_serial]
fn test_profiling_profile_creation() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Create a profile section using the macro
        let section = profile!("test_profile");
        assert!(
            section.is_active(),
            "ProfileSection should be active when profiling is enabled"
        );
        section.end();
    });
}

#[test]
// #[file_serial]
fn test_profiling_nested_profile_sections() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Create an outer profile section
        let outer_section = profile!("outer");
        assert!(outer_section.is_active());

        // Create an inner profile section
        let inner_section = profile!("inner");
        assert!(inner_section.is_active());

        // End the inner section
        inner_section.end();

        // End the outer section
        outer_section.end();
    });
}

// Attribute macro tests

// Using direct profile! macro for integration tests
// because the #[profiled] attribute has path resolution issues
fn simple_profiled_function() -> u32 {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    let _section = thag_profiler::profile!("simple_profiled_function");
    // Simulate some work
    thread::sleep(Duration::from_millis(10));
    42
}

#[test]
// #[file_serial]
fn test_profiling_profiled_attribute() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Call the profiled function
        let result = simple_profiled_function();
        assert_eq!(result, 42);

        // Register this function manually for the test
        register_profiled_function(
            "simple_profiled_function",
            "simple_profiled_function".to_string(),
        );
    });
}

// Async profiling tests

// Using direct macro approach for consistency
async fn async_profiled_function() -> u32 {
    let _section = thag_profiler::profile!("async_profiled_function", async);
    // Simulate some async work
    async_std::task::sleep(Duration::from_millis(10)).await;
    84
}

#[test]
// #[file_serial]
fn test_profiling_async_profiled_function() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Run the async function
        let runtime = async_std::task::block_on(async { async_profiled_function().await });

        assert_eq!(runtime, 84);
    });
}

// macro tests

#[test]
// #[file_serial]
fn test_profiling_profile_macro() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Basic usage
        let section = thag_profiler::profile!("basic_test");
        thread::sleep(Duration::from_millis(5));
        section.end();

        // With type
        let section = thag_profiler::profile!("memory_test", memory);
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Method style - Pass a string since we can't use the method keyword in tests
        let section = thag_profiler::profile!("method_test");
        thread::sleep(Duration::from_millis(5));
        section.end();
    });
}

// Test enabling/disabling profiling

#[test]
// #[file_serial]
fn test_profiling_enable_disable_profiling() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    // Start with profiling disabled
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled(), "Profiling should start disabled");

    // Create a profile section (should be inactive when disabled)
    let section = thag_profiler::profile!("disabled_test");
    assert!(
        !section.is_active(),
        "Section should be inactive when profiling is disabled"
    );
    section.end();

    // Enable profiling
    let _ = enable_profiling(true, ProfileType::Time);
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled after calling enable_profiling"
    );

    // Create a profile section (should be active)
    let section = thag_profiler::profile!("enabled_test");
    assert!(
        section.is_active(),
        "Section should be active when profiling is enabled"
    );
    section.end();

    // Disable profiling again
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled after calling disable_profiling"
    );

    // Create another section (should be inactive again)
    let section = thag_profiler::profile!("disabled_test_again");
    assert!(
        !section.is_active(),
        "Section should be inactive when profiling is disabled again"
    );
    section.end();
}

// Memory profiling test

#[test]
// #[file_serial]
fn test_profiling_memory_profiling() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    // Enable memory profiling
    let _ = enable_profiling(true, ProfileType::Memory);

    // Create a profile section that tracks memory
    let section = thag_profiler::profile!("memory_test", memory);

    // Allocate some memory
    let data = vec![0u8; 1_000_000];

    // Just to prevent the compiler from optimizing away our allocation
    assert_eq!(data.len(), 1_000_000);

    // End the section
    section.end();

    // Disable profiling
    let _ = enable_profiling(false, ProfileType::Time);
}

// Thread-safety test

#[test]
// #[file_serial]
fn test_profiling_profile_section_thread_safety() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Create a profile section that we'll send to another thread
        let section = thag_profiler::profile!("thread_test");

        // Spawn a thread and move the section to it
        let handle = thread::spawn(move || {
            // If ProfileSection is not Send, this won't compile
            assert!(section.is_active());
            section.end();
        });

        // Wait for the thread to finish
        handle.join().unwrap();
    });
}

#[test]
// #[file_serial]
fn test_profiled_function_registration() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled at start"
    );

    // Clear registry or start with a clean state if needed

    // Register a function
    register_profiled_function("test_func", "test_desc".to_string());

    // Dump the registry contents
    let contents = dump_profiled_functions();
    println!("Registry contents: {:?}", contents);

    // Check if it's registered
    let is_registered = is_profiled_function("test_func");
    println!("Is test_func registered? {}", is_registered);
    assert!(is_registered, "test_func should be registered");

    // If the test fails, dump the registry again
    if !is_registered {
        println!(
            "Registry contents after failed assertion: {:?}",
            dump_profiled_functions()
        );
    }
}
