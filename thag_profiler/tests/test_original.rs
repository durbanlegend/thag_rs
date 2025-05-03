use std::{thread, time::Duration};
use thag_profiler::profiling::{
    dump_profiled_functions, is_profiled_function, is_profiling_state_enabled,
    register_profiled_function,
};

#[cfg(feature = "time_profiling")]
use parking_lot::MutexGuard;

#[cfg(feature = "time_profiling")]
use std::panic;

#[cfg(feature = "full_profiling")]
use thag_profiler::{with_allocator, Allocator};

#[cfg(feature = "time_profiling")]
use thag_profiler::{end, profile, profiling::enable_profiling, ProfileType, PROFILING_MUTEX};

#[cfg(feature = "time_profiling")]
struct TestGuard;

#[cfg(feature = "time_profiling")]
impl Drop for TestGuard {
    fn drop(&mut self) {
        let _ = enable_profiling(false, Some(ProfileType::Time));
        eprintln!("TestGuard disabled profiling on drop");
    }
}

// Use this before each test
#[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
fn setup_test() -> MutexGuard<'static, ()> {
    let guard = PROFILING_MUTEX.lock();

    // Reset profiling state completely
    let _ = enable_profiling(false, Some(ProfileType::Time));

    guard
}

// Use this before each test
#[cfg(feature = "full_profiling")]
fn setup_test() -> MutexGuard<'static, ()> {
    let guard = with_allocator(Allocator::System, || PROFILING_MUTEX.lock());

    // Reset profiling state completely
    let _ = with_allocator(Allocator::System, || {
        enable_profiling(false, Some(ProfileType::Time))
    });

    guard
}

#[cfg(feature = "time_profiling")]
fn run_test<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    // Register the crate under a name that the #[profiled] macro will recognize
    // This simulates what would happen if this was using an imported thag_profiler
    register_profiled_function("test_function", "test_description");

    // Explicitly disable profiling first to ensure clean state
    let result = enable_profiling(false, Some(ProfileType::Time));
    eprintln!("Disabling profiling result: {:?}", result);

    // Then enable profiling
    let result = enable_profiling(true, Some(ProfileType::Time));
    eprintln!("Enabling profiling result: {:?}", result);

    // Verify profiling is actually enabled
    let is_enabled = is_profiling_state_enabled();
    eprintln!("Is profiling enabled after explicit enable: {}", is_enabled);
    assert!(
        is_enabled,
        "Profiling should be enabled at the start of run_test"
    );

    // Create guard that will clean up only after the test is done
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
        // Verify profiling is enabled
        eprintln!(
            "Before creating section: is_profiling_enabled = {}",
            is_profiling_state_enabled()
        );

        // Create a profile section using the macro
        profile!("test_profile");
        eprintln!("test_profile={test_profile:#?}");
        eprintln!(
            "After creating section: is_profiling_enabled = {}",
            is_profiling_state_enabled()
        );

        assert!(!test_profile.as_ref().unwrap().detailed_memory());
        end!("test_profile");
    });
}

// Attribute macro tests

// Using direct profile! macro for integration tests
// because the #[profiled] attribute has path resolution issues
fn simple_profiled_function() -> u32 {
    // Get lock and reset state
    #[cfg(feature = "time_profiling")]
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at start"
    );

    thag_profiler::profile!("simple_profiled_function", unbounded, time);
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
    thag_profiler::profile!(
        "async_profiled_function",
        mem_summary,
        time,
        async_fn,
        unbounded
    );
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
        thag_profiler::profile!("basic_test");
        thread::sleep(Duration::from_millis(5));
        end!("basic_test");

        // Method style - Pass a string since we can't use the method keyword in tests
        thag_profiler::profile!("repeat");
        thread::sleep(Duration::from_millis(5));
        end!(repeat);
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

    // Create a profile section
    thag_profiler::profile!("enabled_test");
    assert_eq!(enabled_test.clone().unwrap().file_name(), "test_original");
    end!("enabled_test");
}

// Memory profiling test

#[test]
#[cfg(feature = "full_profiling")]
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
    #[allow(unused_variables)]
    let section = "test_section";
    thag_profiler::profile!(section, mem_summary);

    // Allocate some memory
    let data = vec![0u8; 1_000_000];

    // Just to prevent the compiler from optimizing away our allocation
    assert_eq!(data.len(), 1_000_000);

    // End the section
    end!(section);

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
        thag_profiler::profile!("thread_test", unbounded);

        // Spawn a thread and move the section to it
        let handle = thread::spawn(move || {
            // If ProfileSection is not Send, this won't compile
            assert_eq!(
                thread_test.as_ref().unwrap().fn_name(),
                "test_original::test_profiling_profile_section_thread_safety"
            );
            end!("thread_test");
        });

        // Wait for the thread to finish
        handle.join().unwrap();
        // end!("thread_test");
    });
}

#[test]
fn test_profiling_profiled_function_registration() {
    // Get lock and reset state
    #[cfg(feature = "time_profiling")]
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
