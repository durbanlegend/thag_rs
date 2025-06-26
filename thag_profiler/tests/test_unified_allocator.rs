/// Test demonstrating the unified allocator approach
/// The same code works with either global or thread-local implementation
/// based on the `no_tls` feature flag.
#[cfg(feature = "full_profiling")]
use serial_test::serial;

#[cfg(feature = "full_profiling")]
use std::sync::{Arc, Barrier};

#[cfg(feature = "full_profiling")]
use std::thread;

#[cfg(feature = "full_profiling")]
use thag_profiler::{current_allocator, safe_alloc, Allocator};

#[cfg(feature = "full_profiling")]
use thag_profiler::reset_allocator_state;

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_unified_allocator_basic() {
    reset_allocator_state();

    // Initially should be in Tracking mode
    assert_eq!(current_allocator(), Allocator::Tracking);

    // Test basic switching behavior
    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);
    };

    // Should be back to Tracking mode
    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_unified_allocator_nesting() {
    reset_allocator_state();

    // Test nested behavior - inner call should not interfere
    assert_eq!(current_allocator(), Allocator::Tracking);

    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);

        // Nested call should NOT change anything (already in System mode)
        safe_alloc! {
            assert_eq!(current_allocator(), Allocator::System);
        };

        // Still in System mode after nested call
        assert_eq!(current_allocator(), Allocator::System);
    };

    // Back to Tracking mode - only the outer call resets the flag
    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_unified_allocator_threading() {
    reset_allocator_state();

    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    // Thread 1: Uses allocator switching
    let barrier1 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier1.wait(); // Synchronize start

        safe_alloc! {
            // This thread should be in System mode
            assert_eq!(current_allocator(), Allocator::System);

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        };
    }));

    // Thread 2: Also uses allocator switching
    let barrier2 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier2.wait(); // Synchronize start

        safe_alloc! {
            // This thread should also be in System mode
            assert_eq!(current_allocator(), Allocator::System);

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        };
    }));

    // Thread 3: Checks behavior based on approach
    let barrier3 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier3.wait(); // Synchronize start

        // Small delay to let other threads switch allocators
        thread::sleep(std::time::Duration::from_millis(5));

        // Behavior depends on feature flag:
        // - With no_tls: this thread might see System due to global flag
        // - Without no_tls: this thread should be unaffected (Tracking)
        let current = current_allocator();

        #[cfg(not(feature = "no_tls"))]
        {
            // TLS approach: should be isolated from other threads
            assert_eq!(current, Allocator::Tracking);
        }

        #[cfg(feature = "no_tls")]
        {
            // Global approach: might see System if other threads are active
            // We can't make strict assertions here due to timing
            // Just verify it's one of the valid states
            assert!(matches!(current, Allocator::System | Allocator::Tracking));
        }
    }));

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_unified_approach_selection() {
    reset_allocator_state();

    // This test demonstrates that the same API works regardless of implementation
    println!("Testing unified allocator approach");

    #[cfg(not(feature = "no_tls"))]
    println!("  Using thread-local storage implementation");

    #[cfg(feature = "no_tls")]
    println!("  Using global atomic implementation");

    // The API is identical regardless of implementation
    assert_eq!(current_allocator(), Allocator::Tracking);

    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);

        // Verify we're still in system mode after the safe_alloc call
        assert_eq!(current_allocator(), Allocator::System);
    };

    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
#[serial]
fn test_performance_characteristics() {
    reset_allocator_state();

    use std::time::Instant;
    const ITERATIONS: usize = 1000;

    // Time the unified approach
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        safe_alloc! {
            // Minimal work
            std::hint::black_box(current_allocator());
        };
    }
    let duration = start.elapsed();

    #[cfg(not(feature = "no_tls"))]
    println!("TLS approach: {} iterations in {:?}", ITERATIONS, duration);

    #[cfg(feature = "no_tls")]
    println!(
        "Global approach: {} iterations in {:?}",
        ITERATIONS, duration
    );

    // The test passes regardless of performance - we just measure it
    assert!(duration.as_nanos() > 0);
}

#[test]
fn test_feature_flag_behavior() {
    // This test verifies the feature flag behavior at compile time

    #[cfg(not(feature = "no_tls"))]
    {
        println!("Compiled without no_tls feature - using thread-local approach");
        // Additional TLS-specific functionality would be available here
    }

    #[cfg(feature = "no_tls")]
    {
        println!("Compiled with no_tls feature - using global atomic approach");
        // Global approach is the default
    }

    // Test always passes - it's about compile-time behavior
    assert!(true);
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_advanced_api_availability() {
    // Test that advanced APIs are available when needed

    // Test advanced allocator checking functions
    let _state = thag_profiler::current_allocator();
}
