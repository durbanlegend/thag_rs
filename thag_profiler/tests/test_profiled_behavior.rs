// In thag_profiler/tests/test_profiled_behavior.rs
use thag_profiler::{
    enable_profiling, file_stem_from_path_str, profiled,
    profiling::{get_reg_desc_name, is_profiled_function},
    ProfileType,
};

// Add _test suffix to signal to profiled macro that this is a test function
#[allow(unused_must_use)]
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

// Alternatively, use the explicit 'test' flag
#[allow(unused_must_use)]
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

#[test]
fn test_profiled_behavior() {
    let _ = enable_profiling(true, Some(ProfileType::Memory));

    // Test the synchronous profiled function
    profiled_function_memory();

    // Use async-std to create a runtime for async functions
    // Import the block_on function
    use async_std::task::block_on;

    // Run the async test functions
    block_on(profiled_function_time_test());
    block_on(profiled_function_with_test_flag());

    println!("All tests passed!");
}
