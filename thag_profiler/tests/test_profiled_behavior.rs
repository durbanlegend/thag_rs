/// # Test Suite for #[profiled] Attribute Macro
///
/// This test suite verifies the behavior of the `#[profiled]` attribute macro across different configurations and use cases:
///
/// ```bash
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_profiled_behavior -- --nocapture
/// ```
///
/// ## Key Areas Tested
///
/// 1. **Profile Types**:
///    - Time profiling with `time` flag
///    - Memory profiling with `mem_summary` and `mem_detail` flags
///    - Combined time and memory profiling with `both` flag
///    - Global profile type with `global` flag (uses whatever is set globally)
///    - Default behavior with no flags
///
/// 2. **Special Variants**:
///    - Async function profiling
///    - Test-specific behavior with `test` flag
///    - Legacy parameter syntax (`profile_type = "..."`)
///    - Detailed memory tracking control
///
/// 3. **Profile Attributes**:
///    - Function name and registration
///    - File name determination
///    - Start/end line tracking
///    - Registration in the profiled function registry
///
/// 4. **Combinations and Interactions**:
///    - Different combinations of time and memory profiling
///    - Legacy and modern parameter styles
///    - Synchronous and asynchronous function handling
///
/// ## Test Design Notes
///
/// - All tests are run from a single `test_profiled_behavior` function
/// - A mix of synchronous and asynchronous test functions
/// - Appropriate feature gates for time_profiling and full_profiling
/// - Direct access to profile variables created by the macro
/// - Explicit verification of profile properties
/// - Uses async-std for testing async functions
#[cfg(feature = "time_profiling")]
use thag_profiler::{
    enable_profiling, file_stem_from_path_str, profiled,
    profiling::{get_reg_desc_name, is_profiled_function},
    ProfileType,
};

// Test basic time profiling with _test suffix
#[cfg(feature = "time_profiling")]
#[profiled(time)]
async fn profiled_function_time_test() {
    // Direct access to the 'profile' variable created by the macro
    let profile = profile.as_ref().unwrap();
    let fn_name = profile.fn_name();
    let file_name = profile.file_name();
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(!profile.is_detailed_memory());

    assert_eq!(
        fn_name,
        "test_profiled_behavior::profiled_function_time_test"
    );

    let stack_str = format!("test_profiled_behavior::test_profiled_behavior;{fn_name}");

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.registered_name(), fn_name.to_string());

    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.registered_name(), stack_str);

    assert_eq!(profile.start_line(), None);
    assert_eq!(file_name, "test_profiled_behavior");
    assert_eq!(file_name, file_stem_from_path_str(file!()));
    assert!(is_profiled_function(&stack_str));
    assert_eq!(
        get_reg_desc_name(&stack_str),
        Some(format!("async::{fn_name}"))
    );
}

// Test with explicit test flag
#[cfg(feature = "time_profiling")]
#[profiled(time, test)]
async fn profiled_function_with_test_flag() {
    // Direct access to the 'profile' variable created by the macro
    let profile = profile.as_ref().unwrap();
    let fn_name = profile.fn_name();
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(!profile.is_detailed_memory());

    assert_eq!(
        fn_name,
        "test_profiled_behavior::profiled_function_with_test_flag"
    );
    let stack_str = format!("test_profiled_behavior::test_profiled_behavior;{fn_name}");

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.registered_name(), fn_name.to_string());

    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.registered_name(), stack_str);

    assert!(is_profiled_function(&stack_str));
}

// Test memory profiling with detailed memory flag
#[cfg(feature = "full_profiling")]
#[profiled(mem_detail)]
fn profiled_function_memory() {
    let profile = profile.as_ref().unwrap();
    let fn_name = profile.fn_name();
    let file_name = profile.file_name();
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);
    assert!(profile.is_detailed_memory());

    assert_eq!(fn_name, "test_profiled_behavior::profiled_function_memory");
    let stack_str = format!("test_profiled_behavior::test_profiled_behavior;{fn_name}");
    assert_eq!(profile.registered_name(), stack_str);
    assert_eq!(profile.start_line(), None);
    assert_eq!(file_name, "test_profiled_behavior");
    assert_eq!(file_name, file_stem_from_path_str(file!()));
    assert!(is_profiled_function(&stack_str));
    assert_eq!(get_reg_desc_name(&stack_str), Some(fn_name.to_string()));
}

// Test memory summary without detailed memory
#[cfg(feature = "full_profiling")]
#[profiled(mem_summary)]
fn profiled_function_memory_summary_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_memory_summary_test"
    );
}

// Test both time and memory profiling
#[cfg(feature = "full_profiling")]
#[profiled(both)]
fn profiled_function_both_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_both_test"
    );
}

// Test time and mem_detail together (should result in ProfileType::Both)
#[cfg(feature = "full_profiling")]
#[profiled(time, mem_detail)]
fn profiled_function_time_mem_detail_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(profile.is_detailed_memory());
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_time_mem_detail_test"
    );
}

// Test time and mem_summary together (should result in ProfileType::Both)
#[cfg(feature = "full_profiling")]
#[profiled(time, mem_summary)]
fn profiled_function_time_mem_summary_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_time_mem_summary_test"
    );
}

// Test global flag (uses whatever profile type is set globally)
#[cfg(feature = "time_profiling")]
#[profiled(global)]
fn profiled_function_global_test() {
    let profile = profile.as_ref().unwrap();
    // The profile type should match what was set with enable_profiling
    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.get_profile_type(), ProfileType::Both);

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.get_profile_type(), ProfileType::Time);

    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_global_test"
    );
}

// Test default (no args - should use global)
#[cfg(feature = "time_profiling")]
#[profiled]
fn profiled_function_default_test() {
    let profile = profile.as_ref().unwrap();
    // Should use the global profile type
    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.get_profile_type(), ProfileType::Both);

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.get_profile_type(), ProfileType::Time);

    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_default_test"
    );
}

// Test async function with various flags
#[cfg(feature = "full_profiling")]
#[profiled(both, test)]
async fn profiled_function_both_async_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(!profile.is_detailed_memory());
    let fn_name = profile.fn_name();
    assert_eq!(
        fn_name,
        "test_profiled_behavior::profiled_function_both_async_test"
    );
    let stack_str = format!("test_profiled_behavior::test_profiled_behavior;{fn_name}");
    assert_eq!(profile.registered_name(), stack_str);
    assert_eq!(
        get_reg_desc_name(&stack_str),
        Some(format!("async::{fn_name}"))
    );
}

// Test with legacy param syntax - profile_type
#[cfg(feature = "time_profiling")]
#[profiled(time)]
fn profiled_function_legacy_params_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_legacy_params_test"
    );
}

// Test with legacy detailed_memory param
#[cfg(feature = "time_profiling")]
#[profiled(mem_detail)]
fn profiled_function_detailed_memory_test() {
    let profile = profile.as_ref().unwrap();

    #[cfg(not(feature = "full_profiling"))]
    // Should use the global profile type (Time)
    assert_eq!(profile.get_profile_type(), ProfileType::Time);

    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);

    #[cfg(not(feature = "full_profiling"))]
    assert!(!profile.is_detailed_memory());

    #[cfg(feature = "full_profiling")]
    assert!(profile.is_detailed_memory());
}

// Test with both legacy params
#[cfg(feature = "time_profiling")]
#[profiled(time, mem_detail)]
fn profiled_function_both_legacy_params_test() {
    let profile = profile.as_ref().unwrap();

    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.get_profile_type(), ProfileType::Both);

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.get_profile_type(), ProfileType::Time);

    #[cfg(feature = "full_profiling")]
    assert!(profile.is_detailed_memory());

    #[cfg(not(feature = "full_profiling"))]
    assert!(!profile.is_detailed_memory());
}

#[test]
#[cfg(feature = "time_profiling")]
#[enable_profiling]
fn test_profiled_behavior() {
    use thag_profiler::safe_alloc;

    let closure = || {
        #[cfg(feature = "full_profiling")]
        {
            profiled_function_memory();
            profiled_function_memory_summary_test();
            profiled_function_both_test();
            profiled_function_time_mem_detail_test();
            profiled_function_time_mem_summary_test();
        }

        profiled_function_global_test();
        profiled_function_default_test();
        profiled_function_legacy_params_test();
        profiled_function_detailed_memory_test();
        profiled_function_both_legacy_params_test();

        // Use async-std to create a runtime for async functions
        use async_std::task::block_on;

        // Run the async test functions
        block_on(profiled_function_time_test());
        block_on(profiled_function_with_test_flag());

        #[cfg(feature = "full_profiling")]
        block_on(profiled_function_both_async_test());

        println!("All tests passed!");
    };

    #[cfg(not(feature = "full_profiling"))]
    closure();

    safe_alloc!(closure());
}
