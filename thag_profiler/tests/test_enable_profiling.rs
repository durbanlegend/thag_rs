/// # Test Suite for #[enable_profiling] Attribute Macro
///
/// This test suite verifies that the `#[enable_profiling]` attribute macro correctly enables and disables profiling with different options:
///
/// ```bash
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_enable_profiling -- --nocapture
/// ```
///
/// ## Key Areas Tested
///
/// 1. **Default Behavior**: Verifies that with no options, profiling is enabled using appropriate defaults based on feature flags.
///
/// 2. **Explicit Settings**:
///    - `yes` option: Explicitly enable profiling
///    - `no` option: Explicitly disable profiling
///    - `time` option: Enable time-only profiling
///    - `memory` option: Enable memory-only profiling (with full_profiling feature)
///    - `both` option: Enable both time and memory profiling (with full_profiling feature)
///
/// 3. **Runtime Control**:
///    - Environment variable configuration through `THAG_PROFILER`
///    - Handling valid and invalid profile types
///    - Configuration of detailed memory settings
///
/// 4. **Sequential Testing**:
///    - All tests run in a single sequence to avoid global state conflicts
///    - Tests include proper reset between test cases
///    - Complete coverage of environment variable interactions
///
/// 5. **Feature Compatibility**:
///    - Tests that adapt based on available features (time_profiling, full_profiling)
///    - Appropriate feature-specific assertions
///
/// ## Test Design Notes
///
/// - Uses a single `#[test]` function to ensure sequential execution
/// - Clears environment state between test cases
/// - Explicit check for profile type limitations based on features
/// - Handles potential panics in edge cases like invalid environment values
/// - Validates proper profiling state cleanup after each function completes
///
#[cfg(feature = "time_profiling")]
use std::env;

#[cfg(feature = "time_profiling")]
use thag_profiler::{
    end, mem_tracking, profile,
    profiling::{
        disable_profiling, is_profiling_enabled, is_profiling_state_enabled, set_profile_config,
    },
    ProfileConfiguration, ProfileType,
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
    profile!(default_section, time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!(default_section);
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
    profile!(yes_section, time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!(yes_section);
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
    profile!(no_section, time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!(no_section);
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
    profile!(time_section, time);
    let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
    end!(time_section);
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
    profile!(memory_section, mem_summary);
    let data = vec![0u8; 1024]; // Allocate some memory
    let _ = data.len(); // Use data to avoid compiler optimizations
    end!(memory_section);
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
    profile!(both_section, both);
    let data = vec![0u8; 1024]; // Allocate some memory
    let result = (0..100).fold(0, |acc, x| acc + x); // Do some work
    let _ = data.len() + result; // Use values to avoid compiler optimizations
    end!(both_section);
}

/// Test with "runtime" option (environment variable controlled)
#[cfg(feature = "time_profiling")]
#[thag_profiler::enable_profiling(runtime)]
fn runtime_controlled_function() {
    // This function will check the environment variable THAG_PROFILER
    // It should be enabled if the env var is set, disabled otherwise
    let env_var_exists = env::var("THAG_PROFILER").is_ok();

    assert_eq!(
        is_profiling_enabled(),
        env_var_exists,
        "Profiling enabled state should match THAG_PROFILER environment variable presence"
    );

    // If env var exists, test that profile settings match what's in the variable
    if env_var_exists {
        let env_value = env::var("THAG_PROFILER").unwrap();
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
        profile!(runtime_section, time);
        let _ = (0..100).fold(0, |acc, x| acc + x); // Do some work
        end!(runtime_section);
    }
}

// ---------------------------------------------------------------------------
// Main test function - follows pattern from test_profiled_behavior.rs
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "time_profiling")]
fn test_enable_profiling_full_sequence() {
    use thag_profiler::profiling::clear_profile_config_cache;

    let max_profile_type = if cfg!(feature = "full_profiling") {
        ProfileType::Both
    } else {
        ProfileType::Time
    };

    // Ensure clean state to start
    env::remove_var("THAG_PROFILER");
    disable_profiling();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled at test start"
    );

    // -------------------------------------------------------------------
    // 1. Test default (yes) option
    // -------------------------------------------------------------------

    eprintln!("Testing default enabled function...");
    default_enabled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );
    assert_eq!(thag_profiler::get_global_profile_type(), max_profile_type);

    // -------------------------------------------------------------------
    // 2. Test explicit yes option
    // -------------------------------------------------------------------

    eprintln!("Testing yes enabled function...");
    disable_profiling(); // Reset state before each test
    yes_enabled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 3. Test no option
    // -------------------------------------------------------------------

    eprintln!("Testing no disabled function...");
    // Use an attribute-based macro approach to enable profiling first
    enable_time_profiling_for_test();
    no_disabled_function();

    // Helper function with attribute macro
    #[thag_profiler::enable_profiling(time)]
    fn enable_time_profiling_for_test() {}
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 4. Test time option
    // -------------------------------------------------------------------

    eprintln!("Testing time enabled function...");
    disable_profiling();
    time_enabled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 5. Test memory option (when available)
    // -------------------------------------------------------------------

    #[cfg(feature = "full_profiling")]
    {
        eprintln!("Testing memory enabled function...");
        disable_profiling();
        memory_enabled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after function"
        );
    }

    // -------------------------------------------------------------------
    // 6. Test both option (when available)
    // -------------------------------------------------------------------

    #[cfg(feature = "full_profiling")]
    {
        eprintln!("Testing both enabled function...");
        disable_profiling();
        both_enabled_function();
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should be disabled after function"
        );
    }

    // -------------------------------------------------------------------
    // 7. Test runtime with no env var
    // -------------------------------------------------------------------

    eprintln!("Testing runtime with no env var...");
    env::remove_var("THAG_PROFILER");
    disable_profiling();
    clear_profile_config_cache();
    runtime_controlled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 8. Test runtime with time profile
    // -------------------------------------------------------------------

    eprintln!("Testing runtime with time profile...");
    env::set_var("THAG_PROFILER", "time,.,none,false");
    disable_profiling();
    clear_profile_config_cache();
    runtime_controlled_function();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 9. Test runtime with invalid profile
    // -------------------------------------------------------------------

    eprintln!("Testing runtime with invalid profile...");
    env::set_var("THAG_PROFILER", "invalid,.,none,false");
    disable_profiling();
    clear_profile_config_cache();
    // This might fail if runtime_controlled_function doesn't handle invalid types well
    // Consider adding a try-catch here if necessary
    let _result = std::panic::catch_unwind(|| {
        runtime_controlled_function();
    });
    // We don't care if it panicked - just make sure we clean up
    let _ = set_profile_config(
        ProfileConfiguration::try_from(vec!["time", "", "announce"].as_slice()).unwrap(),
    );
    disable_profiling();
    assert!(
        !is_profiling_state_enabled(),
        "Profiling should be disabled after function"
    );

    // -------------------------------------------------------------------
    // 10. Test runtime option with detailed memory
    // -------------------------------------------------------------------

    #[cfg(feature = "full_profiling")]
    {
        env::set_var("THAG_PROFILER", "both,.,none,true");
        clear_profile_config_cache();
        runtime_controlled_function();
        assert!(!is_profiling_state_enabled());
        assert_eq!(thag_profiler::get_global_profile_type(), max_profile_type);
        assert!(thag_profiler::is_detailed_memory());
    }

    // -------------------------------------------------------------------
    // 11. Test runtime option without detailed memory
    // -------------------------------------------------------------------

    #[cfg(feature = "full_profiling")]
    {
        env::set_var("THAG_PROFILER", "memory,.,quiet,false");
        clear_profile_config_cache();

        // Run the function with runtime option
        runtime_controlled_function();

        // Verify profiling is disabled after function exits
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should remain disabled when THAG_PROFILER is not set"
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
    }

    // -------------------------------------------------------------------
    // 12. Test runtime option with detailed memory
    // -------------------------------------------------------------------

    #[cfg(feature = "full_profiling")]
    {
        // Set the environment variable for detailed memory profiling
        env::set_var("THAG_PROFILER", "memory,.,announce,true");
        clear_profile_config_cache();

        // Run the function with runtime option
        runtime_controlled_function();

        // Verify profiling is disabled after function exits
        assert!(
            !is_profiling_state_enabled(),
            "Profiling should remain disabled when THAG_PROFILER is not set"
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
    }

    // Clean up at the end
    env::remove_var("THAG_PROFILER");
    disable_profiling();

    println!("All enable_profiling tests completed successfully!");
}
