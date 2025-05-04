/// This test file follows the pattern of using a single test function that sequentially runs multiple test cases
/// to avoid concurrency issues with the global state. The tests cover various aspects of the `profiling` module:
///
/// ```bash
/// # Run with time profiling only
/// THAG_PROFILER=time,,announce cargo test --features=time_profiling --test test_profiling -- --nocapture
///
/// # Run with full profiling
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_profiling -- --nocapture
/// ```
///
/// 1. **ProfileCapability** - Tests the capability flags and feature detection
/// 2. **ProfileType** - Tests the different profile types and conversions
/// 3. **ProfileConfiguration** - Tests parsing environment variables and configuration
/// 4. **Debug Level** - Tests different debug levels and string conversions
/// 5. **Global State** - Tests setting and retrieving global profiling state
/// 6. **File Paths** - Tests the generation of profile file paths
/// 7. **Profile Creation** - Tests creating and using Profile objects
/// 8. **Profile Registry** - Tests registering and looking up profiled functions
/// 9. **Stack Trace** - Tests stack trace extraction and cleaning
/// 10. **Profile Stats** - Tests collecting and managing profile statistics
///
/// Each test function focuses on a specific aspect of the profiling system, and the main test function
/// runs them all in sequence with appropriate initialization and cleanup.
///
// Common imports for all test configurations
use std::env;
use std::str::FromStr;
// use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

// Feature-specific imports
#[cfg(feature = "time_profiling")]
use thag_profiler::{
    clear_profile_config_cache, debug_log, disable_profiling, enable_profiling, profiled,
    profiling::{
        build_stack, clean_function_name, extract_path, get_global_profile_type, get_reg_desc_name,
        is_profiled_function, is_profiling_enabled, is_profiling_state_enabled,
        register_profiled_function, DebugLevel, Profile, ProfileCapability, ProfileConfiguration,
        ProfilePaths, ProfileStats, ProfileType,
    },
};

#[cfg(feature = "time_profiling")]
use thag_profiler::profiling::{get_profile_config, get_time_path, set_profile_config};

#[cfg(feature = "full_profiling")]
use thag_profiler::{
    profiling::{get_memory_detail_dealloc_path, get_memory_detail_path, get_memory_path},
    with_allocator, Allocator,
};

#[cfg(feature = "full_profiling")]
use backtrace::Backtrace;

// Set up a mutex for global test resources to avoid conflicts
// static TEST_RESOURCES: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// ---------------------------------------------------------------------------
// Test functions for different profiling module components
// ---------------------------------------------------------------------------

/// Test ProfileCapability and its relationship with ProfileType
#[cfg(feature = "time_profiling")]
fn test_profile_capability() {
    // Create different capabilities
    let none = ProfileCapability::NONE;
    let time = ProfileCapability::TIME;
    let memory = ProfileCapability::MEMORY;
    let both = ProfileCapability::BOTH;

    // Test bit operations
    assert_eq!(both.0, time.0 | memory.0);

    // Test support checking
    assert!(time.supports(ProfileType::Time));
    assert!(!time.supports(ProfileType::Memory));
    assert!(!time.supports(ProfileType::Both));
    assert!(time.supports(ProfileType::None));

    assert!(memory.supports(ProfileType::Memory));
    assert!(!memory.supports(ProfileType::Time));
    assert!(!memory.supports(ProfileType::Both));

    assert!(both.supports(ProfileType::Time));
    assert!(both.supports(ProfileType::Memory));
    assert!(both.supports(ProfileType::Both));

    assert!(none.supports(ProfileType::None));
    assert!(!none.supports(ProfileType::Time));

    // Test conversion from ProfileType
    assert_eq!(
        ProfileCapability::from_profile_type(ProfileType::Time).0,
        time.0
    );
    assert_eq!(
        ProfileCapability::from_profile_type(ProfileType::Memory).0,
        memory.0
    );
    assert_eq!(
        ProfileCapability::from_profile_type(ProfileType::Both).0,
        both.0
    );
    assert_eq!(
        ProfileCapability::from_profile_type(ProfileType::None).0,
        none.0
    );

    // Test intersection
    assert_eq!(both.intersection(ProfileType::Time).0, time.0);
    assert_eq!(both.intersection(ProfileType::Memory).0, memory.0);
    assert_eq!(both.intersection(ProfileType::Both).0, both.0);
    assert_eq!(both.intersection(ProfileType::None).0, none.0);

    // Test what's available in the current build
    let available = ProfileCapability::available();

    #[cfg(all(feature = "time_profiling", not(feature = "full_profiling")))]
    assert_eq!(available.0, time.0);

    #[cfg(feature = "full_profiling")]
    assert_eq!(available.0, both.0);
}

/// Test ProfileType conversions and behavior
#[cfg(feature = "time_profiling")]
fn test_profile_type() {
    // Test FromStr implementation
    assert_eq!(ProfileType::from_str("time"), Some(ProfileType::Time));
    assert_eq!(ProfileType::from_str("memory"), Some(ProfileType::Memory));
    assert_eq!(ProfileType::from_str("both"), Some(ProfileType::Both));
    assert_eq!(ProfileType::from_str("none"), None);
    assert!(ProfileType::from_str("invalid").is_none());

    // Test Display implementation
    assert_eq!(ProfileType::Time.to_string(), "time");
    assert_eq!(ProfileType::Memory.to_string(), "memory");
    assert_eq!(ProfileType::Both.to_string(), "both");
    assert_eq!(ProfileType::None.to_string(), "none");

    // Test conversion from Option<ProfileType> for default values
    let none_option: Option<ProfileType> = None;
    assert_eq!(none_option.unwrap_or_default(), ProfileType::Both); // Default is Both
}

/// Test DebugLevel enum and conversions
#[cfg(feature = "time_profiling")]
fn test_debug_level() {
    // Test FromStr implementation
    assert_eq!(DebugLevel::from_str("none").unwrap(), DebugLevel::None);
    assert_eq!(DebugLevel::from_str("quiet").unwrap(), DebugLevel::Quiet);
    assert_eq!(
        DebugLevel::from_str("announce").unwrap(),
        DebugLevel::Announce
    );
    assert!(DebugLevel::from_str("invalid").is_err());

    // Test case insensitivity
    assert_eq!(DebugLevel::from_str("QUIET").unwrap(), DebugLevel::Quiet);
    assert_eq!(
        DebugLevel::from_str("Announce").unwrap(),
        DebugLevel::Announce
    );

    // Test trimming
    assert_eq!(DebugLevel::from_str(" none ").unwrap(), DebugLevel::None);

    // Test Display implementation
    assert_eq!(DebugLevel::None.to_string(), "none");
    assert_eq!(DebugLevel::Quiet.to_string(), "quiet");
    assert_eq!(DebugLevel::Announce.to_string(), "announce");

    // Test Default implementation
    assert_eq!(DebugLevel::default(), DebugLevel::None);
}

/// Test ProfileConfiguration parsing and behavior
#[cfg(feature = "time_profiling")]
fn test_profile_configuration() {
    // Test parsing from environment string parts
    let config1 = ProfileConfiguration::try_from(["time", "", "quiet"].as_slice()).unwrap();
    assert_eq!(config1.profile_type(), Some(ProfileType::Time));
    assert_eq!(config1.debug_level(), Some(DebugLevel::Quiet));

    let config2 =
        ProfileConfiguration::try_from(["both", "./output", "announce", "true"].as_slice())
            .unwrap();
    assert_eq!(config2.profile_type(), Some(ProfileType::Both));
    assert_eq!(config2.debug_level(), Some(DebugLevel::Announce));
    assert!(config2.is_detailed_memory());

    // Test invalid configurations
    let invalid1 = ProfileConfiguration::try_from(["invalid", "", "quiet"].as_slice());
    assert!(invalid1.is_err());

    // Test that time with detailed_memory is not allowed
    let invalid2 = ProfileConfiguration::try_from(["time", "", "quiet", "true"].as_slice());
    // let config = invalid2.unwrap();
    assert!(
        invalid2.is_err(),
        "time profile with detailed_memory should not be allowed"
    );

    // Test config getters and setters
    let mut config = config1;
    config.set_profile_type(Some(ProfileType::Both));
    assert_eq!(config.profile_type(), Some(ProfileType::Both));

    // Test Display implementation
    let display_str = config.to_string();
    assert!(display_str.contains("Profile Config:"));
    assert!(display_str.contains("Profile Type: Both"));
}

/// Test environment variable parsing for profile configuration
#[cfg(feature = "time_profiling")]
fn test_env_config_parsing() {
    // Save current env var (if any)
    let original_var = env::var("THAG_PROFILER").ok();

    // Test with no env var set
    env::remove_var("THAG_PROFILER");
    clear_profile_config_cache();
    let config1 = get_profile_config();
    assert!(!config1.is_enabled());

    // Test with time profiling
    env::set_var("THAG_PROFILER", "time,,quiet");
    clear_profile_config_cache();
    let config2 = get_profile_config();
    assert!(config2.is_enabled());
    assert_eq!(config2.profile_type(), Some(ProfileType::Time));
    assert_eq!(config2.debug_level(), Some(DebugLevel::Quiet));

    // Test with full config
    env::set_var("THAG_PROFILER", "both,./output,announce,true");
    clear_profile_config_cache();
    let config3 = get_profile_config();
    assert!(config3.is_enabled());
    assert_eq!(config3.profile_type(), Some(ProfileType::Both));
    assert_eq!(config3.debug_level(), Some(DebugLevel::Announce));
    assert!(config3.is_detailed_memory());

    // Test caching behavior
    env::set_var("THAG_PROFILER", "time,,none");
    // Without clearing cache, should get previous config
    let config4 = get_profile_config();
    assert_eq!(config4.profile_type(), Some(ProfileType::Both)); // Still using cached value

    // After clearing cache, should get new config
    clear_profile_config_cache();
    let config5 = get_profile_config();
    assert_eq!(config5.profile_type(), Some(ProfileType::Time)); // Using new value

    // Test set_profile_config
    let mut custom_config = config5;
    custom_config.set_profile_type(Some(ProfileType::Memory));
    let _ = set_profile_config(custom_config);
    let config6 = get_profile_config();
    assert_eq!(config6.profile_type(), Some(ProfileType::Memory));

    // Restore original env var
    if let Some(value) = original_var {
        env::set_var("THAG_PROFILER", value);
    } else {
        env::remove_var("THAG_PROFILER");
    }
    clear_profile_config_cache();
}

/// Test global profiling state management
#[cfg(feature = "time_profiling")]
fn test_global_profiling_state() {
    let closure = || {
        // Save initial state
        let initial_enabled = is_profiling_enabled();
        let _initial_state = is_profiling_state_enabled();
        let initial_type = get_global_profile_type();

        // First disable profiling
        disable_profiling();
        assert!(!is_profiling_enabled());
        assert!(!is_profiling_state_enabled());

        // Test enabling with different profile types
        enable_profiling(true, Some(ProfileType::Time)).expect("Should enable profiling with Time");
        assert!(is_profiling_enabled());
        assert!(is_profiling_state_enabled());
        assert_eq!(get_global_profile_type(), ProfileType::Time);

        // Test memory and both profiling (requires full_profiling)
        // let result = enable_profiling(true, Some(ProfileType::Memory));
        #[cfg(feature = "full_profiling")]
        {
            enable_profiling(true, Some(ProfileType::Memory))
                .expect("Should enable profiling with Memory");
            assert_eq!(get_global_profile_type(), ProfileType::Memory);

            enable_profiling(true, Some(ProfileType::Both))
                .expect("Should enable profiling with Both");
            assert_eq!(get_global_profile_type(), ProfileType::Both);
        }

        #[cfg(not(feature = "full_profiling"))]
        {
            use std::panic::catch_unwind;

            // Run the test, catching any panics to ensure our guard runs
            let result = catch_unwind(|| enable_profiling(true, Some(ProfileType::Memory)));

            // eprintln!("result={result:#?}");
            assert!(result.is_err());
        }

        // #[cfg(not(feature = "full_profiling"))]
        // Test using config default
        enable_profiling(true, None).expect("Should enable profiling with default");

        // Finally, disable again and verify
        disable_profiling();
        assert!(!is_profiling_enabled());

        // Restore initial state if needed for other tests
        if initial_enabled {
            enable_profiling(true, Some(initial_type)).expect("Should restore initial state");
        }
    };

    #[cfg(not(feature = "full_profiling"))]
    closure();

    #[cfg(feature = "full_profiling")]
    with_allocator(Allocator::System, closure);
}

/// Test ProfileFilePaths and file path generation
#[cfg(feature = "time_profiling")]
fn test_profile_file_paths() {
    // Get the paths structure
    let paths = ProfilePaths::get();

    // Test the structure of file paths
    assert!(
        get_time_path().unwrap().ends_with(".folded"),
        "Time path should end with .folded"
    );
    #[cfg(feature = "full_profiling")]
    assert!(
        get_memory_path().unwrap().ends_with("-memory.folded"),
        "Memory path should end with -memory.folded"
    );
    assert!(
        paths.debug_log.ends_with("-debug.log"),
        "Debug log path should end with -debug.log"
    );

    // Test that timestamp format looks correct
    let timestamp = &paths.timestamp;
    assert_eq!(
        timestamp.len(),
        15,
        "Timestamp should be YYYYMMDD-HHMMSS format"
    );
    assert_eq!(
        timestamp.chars().nth(8).unwrap(),
        '-',
        "Timestamp should have a dash at position 8"
    );

    #[cfg(feature = "time_profiling")]
    {
        // Only test these functions with time profiling enabled
        use thag_profiler::profiling::get_time_path;
        let time_path = get_time_path().expect("Should be able to get time path");
        assert!(
            time_path.ends_with(".folded"),
            "Time path should end with .folded"
        );
    }

    #[cfg(feature = "full_profiling")]
    {
        // Test memory-specific paths

        let memory_path = get_memory_path().expect("Should be able to get memory path");
        assert!(
            memory_path.ends_with("-memory.folded"),
            "Memory path should end with -memory.folded"
        );

        let detail_path =
            get_memory_detail_path().expect("Should be able to get memory detail path");
        assert!(
            detail_path.ends_with("-memory_detail.folded"),
            "Detail path should end with -memory_detail.folded"
        );

        let dealloc_path =
            get_memory_detail_dealloc_path().expect("Should be able to get dealloc path");
        assert!(
            dealloc_path.ends_with("-memory_detail_dealloc.folded"),
            "Dealloc path should end with -memory_detail_dealloc.folded"
        );
    }
}

/// Test function registry operations
#[cfg(feature = "time_profiling")]
fn test_function_registry() {
    // Clean registry for test
    let test_prefix = "test_registry_";

    // Register some functions
    let func1 = format!("{test_prefix}func1");
    let func2 = format!("{test_prefix}func2");
    let path1 = format!("{test_prefix}module;{func1}");
    let path2 = format!("{test_prefix}module;{func2}");

    register_profiled_function(&path1, &func1);
    register_profiled_function(&path2, &func2);

    // Verify registrations
    assert!(is_profiled_function(&path1), "func1 should be registered");
    assert!(is_profiled_function(&path2), "func2 should be registered");
    assert!(
        !is_profiled_function("unknown_function"),
        "Unknown function should not be registered"
    );

    // Test descriptive name lookup
    assert_eq!(get_reg_desc_name(&path1), Some(func1.clone()));
    assert_eq!(get_reg_desc_name(&path2), Some(func2.clone()));
    assert_eq!(get_reg_desc_name("unknown_function"), None);

    // let contents = thag_profiler::profiling::dump_profiled_functions();
    // println!("Registry contents: {:#?}", contents);

    // Test extract_path function
    let stack = vec![path1.clone(), "middleware".to_string(), "main".to_string()];
    let path = extract_path(&stack, None);
    assert!(!path.is_empty(), "Path should not be empty");

    // Test build_stack function
    let path_vec = vec!["module".to_string(), func1.clone()];
    let section_name = Some("section".to_string());
    let stack_str = build_stack(&path_vec, section_name.as_ref(), ";");
    assert!(
        stack_str.contains(&func1),
        "Stack should contain function name"
    );
    assert!(
        stack_str.contains("section"),
        "Stack should contain section name"
    );
}

/// Test string cleaning functions
#[cfg(feature = "time_profiling")]
fn test_string_cleaning() {
    // Test clean_function_name
    let mut name1 = "module::function::{{closure}}".to_string();
    assert_eq!(clean_function_name(&mut name1), "module::function");

    let mut name2 = "module::function::h1a2b3c4d".to_string();
    assert_eq!(clean_function_name(&mut name2), "module::function");

    let mut name3 = "module::::function".to_string();
    assert_eq!(clean_function_name(&mut name3), "module::function");

    // Test with both closure and hash
    let mut name4 = "module::function::{{closure}}::h1a2b3c4d".to_string();
    assert_eq!(clean_function_name(&mut name4), "module::function");
}

/// Test Profile creation and operations
#[cfg(feature = "time_profiling")]
fn test_profile_creation() {
    // Set up for profiling
    enable_profiling(true, Some(ProfileType::Time)).expect("Should enable profiling");

    // Create a profile for testing
    let profile = Profile::new(
        Some("test_section"),
        Some("test_function"),
        ProfileType::Time,
        false, // not async
        false, // no detailed memory
        file!(),
        Some(100), // fake start line
        Some(200), // fake end line
    );

    assert!(
        profile.is_some(),
        "Profile should be created when profiling is enabled"
    );

    if let Some(profile) = profile {
        // Verify profile properties
        assert_eq!(profile.get_profile_type(), ProfileType::Time);
        assert!(!profile.is_detailed_memory());
        assert_eq!(profile.section_name(), Some("test_section".to_string()));

        // Test path-related methods
        let path = profile.path();
        assert!(!path.is_empty(), "Path should not be empty");

        // Create another profile with memory detail
        #[cfg(feature = "full_profiling")]
        {
            let memory_profile = Profile::new(
                Some("memory_section"),
                Some("memory_function"),
                ProfileType::Memory,
                false,
                true, // detailed memory
                file!(),
                Some(300),
                Some(400),
            );

            assert!(memory_profile.is_some(), "Memory profile should be created");

            if let Some(profile) = memory_profile {
                assert_eq!(profile.get_profile_type(), ProfileType::Memory);
                assert!(profile.is_detailed_memory());
            }
        }
    }

    // Disable profiling at the end
    disable_profiling();

    // Create a profile when profiling is disabled - should return None
    let no_profile = Profile::new(
        Some("disabled_section"),
        Some("disabled_function"),
        ProfileType::Time,
        false,
        false,
        file!(),
        None,
        None,
    );

    assert!(
        no_profile.is_none(),
        "Profile should not be created when profiling is disabled"
    );
}

/// Test ProfileStats operations
#[cfg(feature = "time_profiling")]
fn test_profile_stats() {
    let mut stats = ProfileStats::default();

    // Record some measurements
    stats.record("function1", Duration::from_micros(100));
    stats.record("function1", Duration::from_micros(200));
    stats.record("function2", Duration::from_micros(150));

    // Verify stats
    assert_eq!(*stats.calls.get("function1").unwrap(), 2);
    assert_eq!(*stats.calls.get("function2").unwrap(), 1);

    assert_eq!(*stats.total_time.get("function1").unwrap(), 300);
    assert_eq!(*stats.total_time.get("function2").unwrap(), 150);
}

/// Test backtrace and stack extraction
#[cfg(feature = "full_profiling")]
fn test_stack_extraction() {
    use thag_profiler::profiling::extract_profile_callstack;

    with_allocator(Allocator::System, || {
        // Create a backtrace
        let mut backtrace = Backtrace::new();

        // eprintln!("backtrace={backtrace:#?}");

        // Extract the call stack
        let callstack = extract_profile_callstack(
            "thag_profiler::mem_tracking::with_allocator", // Our parent function
            &mut backtrace,
        );

        // eprintln!("callstack={callstack:#?}");

        // Verify that the callstack was extracted
        assert!(!callstack.is_empty(), "Callstack should not be empty");

        // The first frame should be from this test function
        assert!(
            callstack[0].contains("test_stack_extraction"),
            "First frame should be the current function"
        );
    });
}

/// Test using a profiled function attribute
#[cfg(feature = "time_profiling")]
#[profiled]
fn test_profiled_function() {
    // This function is marked with the #[profiled] attribute
    // We just need to call it to verify the attribute works
    let closure = || {
        debug_log!("Inside profiled function");

        // Do some work to ensure we register something
        let start = Instant::now();
        #[allow(unused_variables)]
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        let elapsed = start.elapsed();
        debug_log!("Work took {} micros", elapsed.as_micros());

        // In a real test, we'd verify this function was registered
        // For now, we're just verifying the attribute doesn't cause errors
    };

    #[cfg(feature = "full_profiling")]
    with_allocator(Allocator::System, closure);

    #[cfg(not(feature = "full_profiling"))]
    closure();
}

// ---------------------------------------------------------------------------
// Main test function that runs all tests sequentially
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiling_full_sequence() {
    // Save original env var if any
    let original_var = env::var("THAG_PROFILER").ok();

    // Set a known environment for testing
    env::set_var("THAG_PROFILER", "time,,announce");
    clear_profile_config_cache();

    // Run all test functions in sequence
    println!("Starting profiling module tests");

    println!("Testing profile capability...");
    test_profile_capability();

    println!("Testing profile type...");
    test_profile_type();

    println!("Testing debug level...");
    test_debug_level();

    println!("Testing profile configuration...");
    test_profile_configuration();

    println!("Testing environment config parsing...");
    test_env_config_parsing();

    println!("Testing global profiling state...");
    test_global_profiling_state();

    println!("Testing profile file paths...");
    test_profile_file_paths();

    println!("Testing function registry...");
    test_function_registry();

    println!("Testing string cleaning...");
    test_string_cleaning();

    println!("Testing profile creation...");
    test_profile_creation();

    println!("Testing profile stats...");
    test_profile_stats();

    #[cfg(feature = "full_profiling")]
    {
        println!("Testing stack extraction (full_profiling only)...");
        test_stack_extraction();
    }

    println!("Testing profiled function attribute...");
    test_profiled_function();

    // Clean up
    disable_profiling();

    // Restore original env var
    if let Some(value) = original_var {
        env::set_var("THAG_PROFILER", value);
    } else {
        env::remove_var("THAG_PROFILER");
    }
    clear_profile_config_cache();

    println!("All profiling tests passed!");
}
