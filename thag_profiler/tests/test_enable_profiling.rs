//! Tests for the #[enable_profiling] attribute macro.
//!
//! These tests verify that the enable_profiling attribute macro correctly enables
//! and disables profiling with different options.

#[cfg(feature = "time_profiling")]
use std::env;

#[cfg(feature = "time_profiling")]
use thag_profiler::{
    enable_profiling, end, /*, file_stem_from_path_str */
    profile,
    profiling::{
        disable_profiling,
        is_profiling_enabled,
        is_profiling_state_enabled,
        // parse_env_profile_config,
    },
    ProfileType,
};

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
                assert_eq!(
                    thag_profiler::is_detailed_memory(),
                    detailed_memory
                        && (profile_type == "memory" || profile_type == "both")
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
// Tests to run the functions
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_default() {
    // Run the function with default enablement
    default_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );

    // Clean up
    disable_profiling();
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_yes() {
    // Run the function with explicit yes
    yes_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );

    // Clean up
    disable_profiling();
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_no() {
    // First enable profiling to test that 'no' disables it
    let _ = enable_profiling(true, Some(ProfileType::Time));
    assert!(
        is_profiling_enabled(),
        "Profiling should be enabled initially"
    );

    // Run the function with 'no' option
    no_disabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled after function completes"
    );
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_time() {
    // Run the function with time option
    time_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Time,
        "Global profile type should remain as Time after function completes"
    );

    // Clean up
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_memory() {
    // Run the function with memory option
    memory_enabled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Memory,
        "Global profile type should remain as Memory after function completes"
    );

    // Clean up
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_both() {
    // Run the function with both option
    both_enabled_function();

    // Verify profiling is now disabled after function exits
    assert!(
        !is_profiling_enabled(),
        "Profiling should be disabled after function completes"
    );
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function completes"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Both,
        "Global profile type should remain as Both after function completes"
    );

    // Clean up
    disable_profiling();
}

// ---------------------------------------------------------------------------
// Tests for runtime environment variable control
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_runtime_unset() {
    // Ensure the environment variable is not set
    env::remove_var("THAG_PROFILE");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_runtime_time() {
    // Set the environment variable for time profiling
    env::set_var("THAG_PROFILE", "time,.,none,false");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Time,
        "Global profile type should be Time when set in environment variable"
    );

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_runtime_memory() {
    // Set the environment variable for memory profiling
    env::set_var("THAG_PROFILE", "memory,.,quiet,false");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Memory,
        "Global profile type should be Memory when set in environment variable"
    );
    assert!(
        !thag_profiler::is_detailed_memory(),
        "Detailed memory should be disabled"
    );

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_runtime_memory_detailed() {
    // Set the environment variable for detailed memory profiling
    env::set_var("THAG_PROFILE", "memory,.,announce,true");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Memory,
        "Global profile type should be Memory when set in environment variable"
    );
    assert!(
        thag_profiler::is_detailed_memory(),
        "Detailed memory should be enabled"
    );

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_runtime_both() {
    // Set the environment variable for both time and memory profiling
    env::set_var("THAG_PROFILE", "both,.,none,false");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Both,
        "Global profile type should be Both when set in environment variable"
    );

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

#[test]
#[cfg(feature = "full_profiling")]
fn test_enable_profiling_runtime_both_detailed() {
    // Set the environment variable for both time and detailed memory profiling
    env::set_var("THAG_PROFILE", "both,.,announce,true");

    // Run the function with runtime option
    runtime_controlled_function();

    // Verify profiling is disabled after function exits
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should remain disabled when THAG_PROFILE is not set"
    );
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Both,
        "Global profile type should be Both when set in environment variable"
    );
    assert!(
        thag_profiler::is_detailed_memory(),
        "Detailed memory should be enabled"
    );

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

// Test with invalid values in the environment variable
#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_runtime_invalid() {
    // Set the environment variable with invalid profile type
    env::set_var("THAG_PROFILE", "invalid,.,none,false");

    // Run the function with runtime option - should not panic but may log errors
    runtime_controlled_function();

    // Cleanup
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

// Integration test that verifies all options work together
#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_sequence() {
    // Start with profiling disabled
    disable_profiling();
    assert!(!is_profiling_state_enabled());

    // Test default option (yes)
    default_enabled_function();
    assert!(!is_profiling_state_enabled());

    // Test no option (disables profiling)
    no_disabled_function();
    assert!(!is_profiling_state_enabled());

    // Test time option
    time_enabled_function();
    assert!(is_profiling_state_enabled());
    assert_eq!(thag_profiler::get_global_profile_type(), ProfileType::Time);

    // Test runtime option
    env::set_var("THAG_PROFILE", "time,.,none,false");
    runtime_controlled_function();
    assert!(is_profiling_state_enabled());

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}

#[cfg(feature = "full_profiling")]
#[test]
fn test_enable_profiling_full_sequence() {
    // Start with profiling disabled
    disable_profiling();
    assert!(!is_profiling_state_enabled());

    // Test default option with full_profiling (should enable both)
    default_enabled_function();
    assert!(is_profiling_state_enabled());
    assert_eq!(thag_profiler::get_global_profile_type(), ProfileType::Both);

    // Test no option (disables profiling)
    no_disabled_function();
    assert!(!is_profiling_state_enabled());

    // Test memory option
    memory_enabled_function();
    assert!(is_profiling_state_enabled());
    assert_eq!(
        thag_profiler::get_global_profile_type(),
        ProfileType::Memory
    );

    // Test both option
    both_enabled_function();
    assert!(is_profiling_state_enabled());
    assert_eq!(thag_profiler::get_global_profile_type(), ProfileType::Both);

    // Test runtime option with detailed memory
    env::set_var("THAG_PROFILE", "both,.,none,true");
    runtime_controlled_function();
    assert!(is_profiling_state_enabled());
    assert_eq!(thag_profiler::get_global_profile_type(), ProfileType::Both);
    assert!(thag_profiler::is_detailed_memory());

    // Clean up
    env::remove_var("THAG_PROFILE");
    disable_profiling();
}
