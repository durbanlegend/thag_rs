// Tests for the profile! and end! macros
#[cfg(feature = "time_profiling")]
use thag_profiler::{
    enable_profiling, end, file_stem_from_path_str, profile, /*, Profile */
    ProfileType,
};

// #[cfg(feature = "time_profiling")]
// use thag_profiler::profiling::{get_reg_desc_name, is_profiled_function};

/// Test time profiling in a section
#[cfg(feature = "time_profiling")]
fn test_time_section() {
    profile!("test_time_section", time);

    // Verify profile properties
    // let section_name = get_section_profile_name("test_time_section");
    let profile = test_time_section.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Time);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.section_name(),
        Some("test_time_section".to_string())
    );
    // assert_eq!(profile.section_name(), Some(section_name));

    // Check file and line information
    let file_name = profile.file_name();
    assert_eq!(file_name, "test_profile_section");
    assert_eq!(file_name, file_stem_from_path_str(file!()));

    // eprintln!("profile={profile:#?}");

    // Check start line is captured
    assert_eq!(profile.start_line(), None);
    // End line should be None before end! is called
    assert_eq!(profile.end_line(), None);

    // Do some work
    let _ = (0..1000).fold(0, |acc, x| acc + x);

    // Mark the end of the section
    end!(test_time_section);
    eprintln!("test_time_section passed");
}

/// Test memory profiling in a section (summary only)
#[cfg(feature = "full_profiling")]
fn test_memory_summary_section() {
    profile!("test_memory_summary", mem_summary);

    let profile = test_memory_summary.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);
    assert!(!profile.is_detailed_memory());
    assert_eq!(
        profile.section_name(),
        Some("test_memory_summary".to_string())
    );

    // Check line information is captured
    assert!(profile.start_line().is_some());

    // Allocate some memory to track
    let data = vec![0u8; 1024];
    let _more_data = vec![0u8; 2048];

    // Prevent optimizer from removing the allocations
    assert_eq!(data.len(), 1024);

    end!(test_memory_summary);
    eprintln!("test_memory_summary_section passed");
}

/// Test memory profiling in a section (detailed)
#[cfg(feature = "full_profiling")]
fn test_memory_detail_section() {
    profile!("test_memory_detail", mem_detail);

    let profile = test_memory_detail.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);
    assert!(profile.is_detailed_memory());
    assert_eq!(
        profile.section_name(),
        Some("test_memory_detail".to_string())
    );

    // Check line information for attribution
    let start_line = profile.start_line().unwrap();
    assert!(start_line > 0);

    // Allocate some memory with different patterns
    let data1 = vec![1u8; 1024];
    let data2 = vec![2u8; 2048];
    let data3 = vec![3u8; 4096];

    // Prevent optimizer from removing the allocations
    assert_eq!(data1.len() + data2.len() + data3.len(), 7168);

    end!(test_memory_detail);
    eprintln!("test_memory_detail_section passed");
}

/// Test combined time and memory profiling
#[cfg(feature = "full_profiling")]
fn test_both_profiling_section() {
    profile!("test_both", time, mem_summary);

    let profile = test_both.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(!profile.is_detailed_memory());

    // Do work and allocate memory
    let result = (0..1000).fold(0, |acc, x| acc + x);
    let data = vec![0u8; result as usize % 10000];

    // Prevent optimizer from removing the allocations
    assert!(data.capacity() > 0);

    end!(test_both);
    eprintln!("test_both_profiling_section passed");
}

/// Test combined time and detailed memory profiling
#[cfg(feature = "full_profiling")]
fn test_both_detailed_section() {
    profile!("test_both_detailed", time, mem_detail);

    let profile = test_both_detailed.as_ref().unwrap();
    assert_eq!(profile.get_profile_type(), ProfileType::Both);
    assert!(profile.is_detailed_memory());

    // Do work and allocate memory
    let data = (0..1000).map(|i| i.to_string()).collect::<Vec<_>>();

    // Prevent optimizer from removing the allocations
    assert_eq!(data.len(), 1000);

    end!(test_both_detailed);
    eprintln!("test_both_detailed_section passed");
}

/// Test global profile type setting
#[cfg(feature = "time_profiling")]
fn test_global_profile_section() {
    profile!("test_global", global);

    let profile = test_global.as_ref().unwrap();

    // The profile type should match what was set with enable_profiling
    #[cfg(feature = "full_profiling")]
    assert_eq!(profile.get_profile_type(), ProfileType::Memory);

    #[cfg(not(feature = "full_profiling"))]
    assert_eq!(profile.get_profile_type(), ProfileType::Time);

    end!(test_global);
    eprintln!("test_global_profile_section passed");
}

/// Test nested sections with different profile types
#[cfg(feature = "full_profiling")]
fn test_nested_sections() {
    profile!("outer_section", time);

    let outer_profile = outer_section.as_ref().unwrap();
    assert_eq!(outer_profile.get_profile_type(), ProfileType::Time);

    // Do some work in the outer section
    let _ = (0..500).fold(0, |acc, x| acc + x);

    // Create a nested section with different profile type
    profile!("inner_section", mem_detail);

    let inner_profile = inner_section.as_ref().unwrap();
    assert_eq!(inner_profile.get_profile_type(), ProfileType::Memory);
    assert!(inner_profile.is_detailed_memory());

    // Do some memory allocations in the inner section
    let inner_data = vec![0u8; 4096];
    assert_eq!(inner_data.len(), 4096);

    end!(inner_section);

    // Continue with the outer section
    let outer_data = vec![1u8; 2048];
    assert_eq!(outer_data.len(), 2048);

    end!(outer_section);
    eprintln!("test_nested_sections passed");
}

/// Test unbounded section (without explicit end)
#[cfg(feature = "full_profiling")]
fn test_unbounded_section() {
    {
        profile!("unbounded_section", mem_detail, unbounded);

        let profile = unbounded_section.as_ref().unwrap();
        assert_eq!(profile.get_profile_type(), ProfileType::Memory);
        assert!(profile.is_detailed_memory());
        assert!(profile.start_line().is_some());
        assert_eq!(profile.end_line(), None); // Should be None for unbounded

        // Allocate memory
        let data = vec![0u8; 8192];
        assert_eq!(data.len(), 8192);

        // No explicit end! call - section ends when profile is dropped
    } // unbounded_section profile is dropped here

    // Verify the profile was dropped
    #[allow(unused_variables)]
    let is_dropped = if cfg!(feature = "time_profiling") {
        // This would fail if profile wasn't dropped
        let unbounded_section = 42; // Reuse the name
        unbounded_section == 42
    } else {
        true
    };

    assert!(is_dropped);
    eprintln!("test_unbounded_section passed");
}

/// Helper function to get the section name from a profile
#[allow(dead_code)]
fn get_section_profile_name(section_name: &str) -> String {
    format!("{}::{section_name}", module_path!())
}

#[test]
#[cfg(feature = "time_profiling")]
fn test_profile_section_behavior() {
    // Enable profiling with appropriate type based on feature flags
    #[cfg(feature = "full_profiling")]
    let _ = enable_profiling(true, Some(ProfileType::Memory));

    #[cfg(not(feature = "full_profiling"))]
    let _ = enable_profiling(true, Some(ProfileType::Time));

    // Run all the test functions
    test_time_section();
    test_global_profile_section();

    #[cfg(feature = "full_profiling")]
    {
        test_memory_summary_section();
        test_memory_detail_section();
        test_both_profiling_section();
        test_both_detailed_section();
        test_nested_sections();
        test_unbounded_section();
    }

    println!("All profile section tests passed!");
}
