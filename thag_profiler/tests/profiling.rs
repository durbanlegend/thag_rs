// use serial_test::file_serial;
// use serial_test::is_locked_serially;
use std::{
    sync::{Mutex, MutexGuard},
    thread,
    time::Duration,
};
use thag_profiler::{
    profiling::{
        dump_profiled_functions, enable_profiling, is_profiled_function,
        is_profiling_state_enabled, register_profiled_function,
    },
    ProfileType,
};

#[cfg(feature = "time_profiling")]
use std::panic;

#[cfg(feature = "time_profiling")]
use thag_profiler::profile;

// Static mutex for test synchronization
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(feature = "time_profiling")]
struct TestGuard;

#[cfg(feature = "time_profiling")]
impl Drop for TestGuard {
    fn drop(&mut self) {
        let _ = enable_profiling(false, Some(ProfileType::Time));
        eprintln!("TestGuard disabled profiling on drop");
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
    let _ = enable_profiling(false, Some(ProfileType::Time));

    // Reset any other global state here
    // ...

    guard
}

#[cfg(feature = "time_profiling")]
fn run_test<T>(test: T) -> ()
where
    T: FnOnce() + panic::UnwindSafe,
{
    // Register the crate under a name that the #[profiled] macro will recognize
    // This simulates what would happen if this was using an imported thag_profiler
    register_profiled_function("test_function", "test_description");

    // Explicitly disable profiling first to ensure clean state
    let _ = enable_profiling(false, Some(ProfileType::Time));

    // Then enable profiling
    let _ = enable_profiling(true, Some(ProfileType::Time));

    // Verify profiling is actually enabled
    assert!(
        is_profiling_state_enabled(),
        "Profiling should be enabled at the start of run_test"
    );

    // Create guard that will clean up even if test panics
    let _guard = TestGuard;

    // Register the crate under a name that the #[profiled] macro will recognize
    // This simulates what would happen if this was using an imported thag_profiler
    register_profiled_function("test_function", "test_description");

    // Explicitly disable profiling first to ensure clean state
    let _ = enable_profiling(false, Some(ProfileType::Time));

    // Run the test, catching any panics to ensure our guard runs
    let result = panic::catch_unwind(test);

    // Re-throw any panic after our guard has cleaned up
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// Basic profiling tests

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_profile_creation() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
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
#[cfg(feature = "time_profiling")]
fn test_profiling_nested_profile_sections() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
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
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    let _section = thag_profiler::profile!("simple_profiled_function");
    // Simulate some work
    thread::sleep(Duration::from_millis(10));
    42
}

#[test]
fn test_profiling_profiled_attribute() {
    let result = simple_profiled_function();
    assert_eq!(result, 42);
}

// Async profiling tests

// Using direct macro approach for consistency
#[cfg(feature = "time_profiling")]
async fn async_profiled_function() -> u32 {
    let _section = thag_profiler::profile!("async_profiled_function", async);
    // Simulate some async work
    smol::Timer::after(Duration::from_millis(500)).await;
    84
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_async_profiled_function() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Run the async function
        let runtime = smol::block_on(async { async_profiled_function().await });

        assert_eq!(runtime, 84);
    });
}

// macro tests

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_profile_macro() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    run_test(|| {
        // Basic usage
        let section = thag_profiler::profile!("basic_test");
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Method style - Pass a string since we can't use the method keyword in tests
        let section = thag_profiler::profile!("repeat");
        thread::sleep(Duration::from_millis(5));
        section.end();
    });
}

// Test enabling/disabling profiling

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_create_section() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    // Enable profiling
    let _ = enable_profiling(true, Some(ProfileType::Time));
    assert!(
        is_profiling_state_enabled(),
        "Profiling should be enabled after calling enable_profiling"
    );

    // Create a profile section (should be active)
    let section = thag_profiler::profile!("enabled_test");
    assert!(
        section.is_active(),
        "Section should be active when profiling is enabled"
    );
    section.end();
}

// Memory profiling test

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_full_profiling() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    // Enable memory profiling
    let _ = enable_profiling(true, Some(ProfileType::Memory));

    assert!(
        is_profiling_state_enabled(),
        "Profiling should be enabled after calling enable_profiling"
    );

    // Create a profile section that tracks memory
    let section = thag_profiler::profile!("test_section");

    // Allocate some memory
    let data = vec![0u8; 1_000_000];

    // Just to prevent the compiler from optimizing away our allocation
    assert_eq!(data.len(), 1_000_000);

    // End the section
    section.end();

    // Disable profiling
    let _ = enable_profiling(false, Some(ProfileType::Time));
}

// Thread-safety test

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_profile_section_thread_safety() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
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
fn test_profiling_profiled_function_registration() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    // Clear registry or start with a clean state if needed

    // Register a function
    register_profiled_function("test_func", "test_desc");

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
