// In thag_profiler/tests/test_profiled_behavior.rs
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
    assert_eq!(profile.registered_name(), fn_name);
    assert_eq!(profile.start_line(), None);
    assert_eq!(file_name, "test_profiled_behavior");
    assert_eq!(file_name, file_stem_from_path_str(file!()));
    assert!(is_profiled_function(fn_name));
    assert_eq!(
        get_reg_desc_name(fn_name),
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
    assert_eq!(profile.registered_name(), fn_name);
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
    assert_eq!(profile.registered_name(), fn_name);
    assert_eq!(profile.start_line(), None);
    assert_eq!(file_name, "test_profiled_behavior");
    assert_eq!(file_name, file_stem_from_path_str(file!()));
    assert!(is_profiled_function(fn_name));
    assert_eq!(get_reg_desc_name(fn_name), Some(fn_name.to_string()));
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
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);

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
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);

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
    assert_eq!(
        profile.fn_name(),
        "test_profiled_behavior::profiled_function_both_async_test"
    );
    assert_eq!(
        get_reg_desc_name(profile.fn_name()),
        Some(format!("async::{}", profile.fn_name()))
    );
}

// Test with legacy param syntax - profile_type
#[cfg(feature = "time_profiling")]
#[profiled(profile_type="time")]
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
#[profiled(detailed_memory=true)]
fn profiled_function_detailed_memory_test() {
    let profile = profile.as_ref().unwrap();
    // Should use the global profile type (Time)
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(profile.is_detailed_memory());
}

// Test with both legacy params
#[cfg(feature = "time_profiling")]
#[profiled(profile_type="time", detailed_memory=true)]
fn profiled_function_both_legacy_params_test() {
    let profile = profile.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(profile.is_detailed_memory());
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_profiled_behavior() {
    #[cfg(feature = "full_profiling")]
    let _ = enable_profiling(true, Some(ProfileType::Memory));

    #[cfg(not(feature = "full_profiling"))]
    let _ = enable_profiling(true, Some(ProfileType::Time));

    // Test the synchronous profiled functions
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
}