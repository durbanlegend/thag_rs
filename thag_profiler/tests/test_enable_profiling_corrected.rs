//! Tests for the #[enable_profiling] attribute macro.
//!
//! These tests verify that the enable_profiling attribute macro correctly enables
//! and disables profiling with different options.

#[cfg(feature = "time_profiling")]
use std::env;

#[cfg(feature = "time_profiling")]
use thag_profiler::{
    enable_profiling, end, file_stem_from_path_str, profile,
    profiling::{disable_profiling, is_profiling_enabled, is_profiling_state_enabled},
    ProfileType,
};

#[cfg(feature = "time_profiling")]
use thag_profiler::reset_profiling_config_for_tests;

// ---------------------------------------------------------------------------
// Test functions with enable_profiling attribute
// ---------------------------------------------------------------------------

/// Test with default option (yes)
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling]
fn default_enabled_function() {
    assert!(is_profiling_enabled(), "Profiling should be enabled");

    // Verify the global profile type
    #[cfg(feature = "full_profiling")]
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Both,
        "Default global profile type should be Both with full_profiling feature"
    );

    #[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Time,
        "Default global profile type should be Time with time_profiling feature"
    );

    // Test section profiling works when enabled
    profile!("default_section", time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!("default_section");
}

/// Test with explicit "yes" option
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling(yes)]
fn yes_enabled_function() {
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled with 'yes' option"
    );

    // Test section profiling works when enabled
    profile!("yes_section", time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!("yes_section");
}

/// Test with "no" option (disabled)
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling(no)]
fn no_disabled_function() {
    // Profiling should be disabled when using 'no'
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled with 'no' option"
    );

    // Section profiling shouldn't create active profiles when disabled
    profile!("no_section", time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!("no_section");
    // No assertions needed - if profiling is incorrectly enabled, this would still work but not fail the test
}

/// Test with "time" option
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling(time)]
fn time_enabled_function() {
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled with 'time' option"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Time,
        "Global profile type should be Time"
    );

    // Test section profiling works when enabled for time
    profile!("time_section", time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!("time_section");
}

/// Test with "memory" option
#[cfg(feature = "full_profiling")]
#[thag_profiler::enable_profiling(memory)]
fn memory_enabled_function() {
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled with 'memory' option"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Memory,
        "Global profile type should be Memory"
    );

    // Test section profiling works when enabled for memory
    profile!("memory_section", mem_summary);
    let data = vec![0u8; 1024]; // Allocate some memory
    let _ = data.len(); // Use data to avoid compiler optimizations
    end!("memory_section");
}

/// Test with "both" option (time and memory)
#[cfg(feature = "full_profiling")]
#[thag_profiler::enable_profiling(both)]
fn both_enabled_function() {
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled with 'both' option"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Both,
        "Global profile type should be Both"
    );

    // Test section profiling works for both time and memory
    profile!("both_section", both);
    let data = vec![0u8; 1024]; // Allocate some memory
    let result = (0..100).fold(0, |acc, x| acc + x); // Do some work
    let _ = data.len() + result; // Use values to avoid compiler optimizations
    end!("both_section");
}

/// Test with "runtime" option (environment variable controlled)
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling(runtime)]
fn runtime_controlled_function() {
    // This function will check the environment variable THAG_PROFILE
    // It should be enabled if the env var is set, disabled otherwise
    let env_var_exists = env::var("THAG_PROFILE").is_ok();

    assert_eq!(
        is_profiling_enabled(),
        env_var_exists,
        "Profiling enabled state should match THAG_PROFILE environment variable presence"
    );

    // If env var exists, test that profile settings match what's in the variable
    if env_var_exists {
        let env_value = env::var("THAG_PROFILE").unwrap();
        let parts: Vec<&str> = env_value.split(',').collect();

        if !parts.is_empty() {
            let profile_type = parts[0].trim();

            match profile_type {
                "time" => assert_eq!(
                    thag_profiler::get_global_profile_type(),
                    ProfileType::Time,
                    "Global profile type should be Time from env var"
                ),
                "memory" => {
                    #[cfg(feature = "full_profiling")]
                    assert_eq!(
                        thag_profiler::get_global_profile_type(),
                        ProfileType::Memory,
                        "Global profile type should be Memory from env var"
                    );

                    #[cfg(not(feature = "full_profiling"))]
                    assert_eq!(
                        thag_profiler::get_global_profile_type(),
                        ProfileType::Time,
                        "Global profile type should fall back to Time when Memory requested but full_profiling not enabled"
                    );
                }
                "both" => {
                    #[cfg(feature = "full_profiling")]
                    assert_eq!(
                        thag_profiler::get_global_profile_type(),
                        ProfileType::Both,
                        "Global profile type should be Both from env var"
                    );

                    #[cfg(not(feature = "full_profiling"))]
                    assert_eq!(
                        thag_profiler::get_global_profile_type(),
                        ProfileType::Time,
                        "Global profile type should fall back to Time when Both requested but full_profiling not enabled"
                    );
                }
                _ => {}
            }

            // Test detailed memory flag if specified in 4th position
            if parts.len() >= 4 && !parts[3].trim().is_empty() {
                let detailed_memory = parts[3].trim().parse::<bool>().unwrap_or(false);
                let profile_type_str = parts[0].trim(); // Get profile type again to ensure correct scope
                assert_eq!(
                    thag_profiler::is_detailed_memory(),
                    detailed_memory
                        && (profile_type_str == "memory" || profile_type_str == "both")
                        && cfg!(feature = "full_profiling"),
                    "Detailed memory setting should match env var"
                );
            }
        }
    }

    // Test section profiling if profiling is enabled
    if is_profiling_enabled() {
        profile!("runtime_section", time);
        let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
        end!("runtime_section");
    }
}

// ---------------------------------------------------------------------------
// Main test function - follows pattern from test_profiled_behavior.rs
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_attribute() {
    // Reset profiling config at the start of the test to ensure we pick up current env vars
    reset_profiling_config_for_tests();

    // Ensure profiling is disabled at the start
    disable_profiling();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at test start"
    );

    // -------------------------------------------------------------------
    // Test default option (yes)
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    default_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after default function completes"
    );

    // -------------------------------------------------------------------
    // Test explicit yes option
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    yes_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after yes function completes"
    );

    // -------------------------------------------------------------------
    // Test time option
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    time_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after time function completes"
    );

    // -------------------------------------------------------------------
    // Test no option
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    // First enable profiling to verify that 'no' doesn't change its state
    let _ = enable_profiling(true, Some(ProfileType::Time));
    assert!(
        is_profiling_state_enabled(),
        "Profiling should be enabled before no function"
    );

    no_disabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after no function completes"
    );

    // -------------------------------------------------------------------
    // Test memory and both options (when full_profiling is available)
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    #[cfg(feature = "full_profiling")]
    {
        // Test memory option
        memory_enabled_function();

        // Verify profiling is disabled after function exits
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after memory function completes"
        );

        // Test both option
        both_enabled_function();

        // Verify profiling is disabled after function exits
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after both function completes"
        );
    }

    // -------------------------------------------------------------------
    // Test runtime option with different environment variable settings
    // -------------------------------------------------------------------
    reset_profiling_config_for_tests();

    // Test with no environment variable set
    env::remove_var("THAG_PROFILE");
    runtime_controlled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after runtime function with no env var"
    );

    // Test with time profile type
    env::set_var("THAG_PROFILE", "time,.,none,false");
    // We don't need #[cfg(test)] here because the integration test already enables
    // the testing feature which makes this function available
    reset_profiling_config_for_tests();
    runtime_controlled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after runtime function with time env var"
    );
    env::remove_var("THAG_PROFILE");

    // Test memory and both profile types when full_profiling is available
    #[cfg(feature = "full_profiling")]
    {
        // Test with memory profile type
        env::set_var("THAG_PROFILE", "memory,.,quiet,false");
        runtime_controlled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after runtime function with memory env var"
        );
        env::remove_var("THAG_PROFILE");

        // Test with detailed memory profile type
        env::set_var("THAG_PROFILE", "memory,.,announce,true");
        runtime_controlled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after runtime function with detailed memory env var"
        );
        env::remove_var("THAG_PROFILE");

        // Test with both profile type
        env::set_var("THAG_PROFILE", "both,.,none,false");
        runtime_controlled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after runtime function with both env var"
        );
        env::remove_var("THAG_PROFILE");

        // Test with both and detailed memory profile type
        env::set_var("THAG_PROFILE", "both,.,announce,true");
        runtime_controlled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after runtime function with both detailed env var"
        );
        env::remove_var("THAG_PROFILE");
    }

    // Test with invalid profile type
    env::set_var("THAG_PROFILE", "invalid,.,none,false");
    runtime_controlled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after runtime function with invalid env var"
    );
    env::remove_var("THAG_PROFILE");

    println!("All enable_profiling attribute tests passed!");
}
