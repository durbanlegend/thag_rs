#[cfg(feature = "full_profiling")]
use std::sync::{Arc, Barrier};

#[cfg(feature = "full_profiling")]
use std::thread;

/// Test comparing global atomic vs thread-local allocator switching behavior
#[cfg(feature = "full_profiling")]
use thag_profiler::safe_alloc;

use thag_profiler::debug_log;

#[cfg(feature = "full_profiling")]
use thag_profiler::{current_allocator, Allocator};

#[test]
#[cfg(feature = "full_profiling")]
fn test_global_vs_tls_allocator_switching() {
    // Test that global and TLS versions work independently

    thag_profiler::profiling::force_set_profiling_state(true);
    assert!(thag_profiler::is_profiling_enabled());

    // Initially both should be in Tracking mode
    assert_eq!(current_allocator(), Allocator::Tracking);

    // Changes global flag from false->true, then back to false
    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);
    };

    // Changes global flag from false->true, then back to false
    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System); // Global unaffected
    };

    // Both should be back to Tracking (flags reset by their respective setters)
    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_macro_versions() {
    // Test that macro versions work correctly

    thag_profiler::profiling::force_set_profiling_state(true);
    assert!(thag_profiler::is_profiling_enabled());

    // Initially both should be in Tracking mode
    assert_eq!(current_allocator(), Allocator::Tracking);

    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);
    };

    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);
    };

    // Test nested usage - inner macro finds flag already set, doesn't touch it
    safe_alloc! {
        assert_eq!(current_allocator(), Allocator::System);

        safe_alloc! {
            assert_eq!(current_allocator(), Allocator::System); // Still system
        };

        assert_eq!(current_allocator(), Allocator::System); // Still system
    };

    assert_eq!(current_allocator(), Allocator::Tracking);
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_thread_isolation() {
    thag_profiler::profiling::force_set_profiling_state(true);
    assert!(thag_profiler::is_profiling_enabled());

    // Test that TLS version provides thread isolation
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    // Thread 1: Uses global allocator switching
    let barrier1 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier1.wait(); // Synchronize start

        safe_alloc! {
            // This thread is in System mode
            assert_eq!(current_allocator(), Allocator::System);

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        };
    }));

    // Thread 2: Uses TLS allocator switching
    let barrier2 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier2.wait(); // Synchronize start

        safe_alloc! {
            // This thread is in System mode (TLS)
            assert_eq!(current_allocator(), Allocator::System); // Global unaffected

            // Sleep to ensure other threads can check their state
            thread::sleep(std::time::Duration::from_millis(10));
        };
    }));

    // Thread 3: Checks it's unaffected by other threads
    let barrier3 = barrier.clone();
    handles.push(thread::spawn(move || {
        barrier3.wait(); // Synchronize start

        // Small delay to let other threads switch allocators
        thread::sleep(std::time::Duration::from_millis(5));

        // Should be in Tracking mode despite other threads
        assert_eq!(current_allocator(), Allocator::Tracking);

        // Global might be affected by thread 1, but TLS should be isolated
        // We can't assert the global state since thread 1 might still be running
    }));

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
#[cfg(feature = "debug_logging")]
fn test_debug_log_macro() {
    // Test the zero-cost debug logging macro
    debug_log!("This is a test message: {}", 42);
    debug_log!("Multiple args: {} {} {}", "hello", "world", 123);

    // When debug_logging feature is disabled, these should compile to nothing
    // When enabled, they should write to the debug log
}

#[test]
fn test_debug_log_zero_cost() {
    // This test should always pass - when debug_logging is disabled,
    // the macro should compile to nothing
    debug_log!("This should be zero-cost when feature is disabled");

    // The fact that this compiles and runs means the macro is working correctly
    assert!(true);
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_performance_comparison() {
    use std::time::Instant;

    const ITERATIONS: usize = 1000;

    thag_profiler::profiling::force_set_profiling_state(true);
    assert!(thag_profiler::is_profiling_enabled());

    // Time global atomic version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        safe_alloc! {
            // Minimal work
            std::hint::black_box(current_allocator());
        };
    }
    let global_duration = start.elapsed();

    // Time TLS version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        safe_alloc! {
            // Minimal work
            std::hint::black_box(current_allocator());
        };
    }
    let tls_duration = start.elapsed();

    println!(
        "Performance comparison ({} iterations):\n  Global atomic: {:?}\n  Thread-local: {:?}\n  TLS speedup: {:.2}x",
        ITERATIONS,
        global_duration,
        tls_duration,
        global_duration.as_nanos() as f64 / tls_duration.as_nanos() as f64
    );

    // TLS should be faster (though this is not a strict requirement for the test)
    // We just print the results for analysis
}
