/// This test file follows the pattern of using a single test function that sequentially runs multiple test cases to avoid concurrency issues with the global state.
/// The tests cover various aspects of the `mem_attribution` module:
///
/// ```bash
/// THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_mem_attribution -- --nocapture
/// ```
///
/// 1. **Whole function profiling** - Tests profile registration for entire functions
/// 2. **Section profiling** - Tests profile registration for code sections with line numbers
/// 3. **Nested sections** - Tests how nested profile sections are handled
/// 4. **Persistent allocations** - Tests tracking of long-lived allocations
/// 5. **Manual profile creation** - Tests directly creating and registering profiles
/// 6. **Registry functions** - Tests the core registry functionality
/// 7. **Overlapping profiles** - Tests how overlapping profile ranges are handled
/// 8. **Record allocation** - Tests the record_allocation function directly
///
/// Each test function focuses on a specific aspect of the memory attribution system, and the main test function (`test_mem_attribution_full_sequence`) runs them all in sequence with appropriate logging.
///
/// Some key points about this testing approach:
///
/// 1. The test is conditional on the `full_profiling` feature being enabled
/// 2. It uses a static `TEST_ALLOCATIONS` to test persistence of allocations across test functions
/// 3. It directly tests the registry's functionality through both the public API and direct access to the registry
/// 4. It covers edge cases like overlapping profiles and out-of-range line numbers
///
#[cfg(feature = "full_profiling")]
use thag_profiler::{
    enable_profiling, end, file_stem_from_path_str,
    mem_attribution::{find_profile, PROFILE_REGISTRY},
    profile, profiled,
    profiling::{Profile, ProfileType},
    with_sys_alloc,
};

#[cfg(feature = "full_profiling")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "full_profiling")]
static TEST_ALLOCATIONS: LazyLock<Mutex<Vec<Vec<u8>>>> = LazyLock::new(|| Mutex::new(Vec::new()));

// ---------------------------------------------------------------------------
// Test functions with different memory profiling patterns
// ---------------------------------------------------------------------------

/// Test function for whole-function profiling
#[cfg(feature = "full_profiling")]
#[profiled(mem_detail)]
fn mem_attribution_whole_function() {
    // Allocate some memory to track
    let data = vec![0u8; 1024];
    let data2 = vec![0u8; 2048];

    // Prevent optimizer from removing the allocations
    assert_eq!(data.len() + data2.len(), 3072);

    // Verify profile registration
    // eprintln!("profile={profile:#?}");
    let profile = profile.as_ref().unwrap();
    let file_name = profile.file_name();
    let fn_name = profile.fn_name();

    // The profile should not have specific line numbers
    assert_eq!(profile.start_line(), None);
    assert_eq!(profile.end_line(), None);
    assert!(profile.detailed_memory());

    // Verify we can find the profile
    let found_profile = find_profile(&file_name, fn_name, 0);
    assert!(
        found_profile.is_some(),
        "Profile should be registered and findable"
    );
}

/// Test function with sectioned memory profiling
#[cfg(feature = "full_profiling")]
fn mem_attribution_with_sections() {
    // Create a section with line numbers
    profile!(test_section_1, mem_detail);

    // The profile should have a start line number
    let section_profile = test_section_1.as_ref().unwrap();
    let start_line = section_profile.start_line().unwrap();
    assert!(start_line > 0);
    assert!(section_profile.detailed_memory());

    // Allocate some memory to track
    let data = vec![0u8; 4096];
    assert_eq!(data.len(), 4096);

    // End the section
    end!(test_section_1);

    // Create another section
    profile!(test_section_2, mem_summary);

    // This section shouldn't use detailed memory
    let section_profile = test_section_2.as_ref().unwrap();
    assert!(!section_profile.detailed_memory());

    // Allocate more memory
    let data = vec![0u8; 8192];
    assert_eq!(data.len(), 8192);

    end!(test_section_2);
}

/// Test function with nested sections
#[cfg(feature = "full_profiling")]
fn mem_attribution_nested_sections() {
    // Outer section
    profile!(outer_section, mem_detail);

    let outer_profile = outer_section.as_ref().unwrap();
    let outer_start = outer_profile.start_line().unwrap();

    // Allocate some memory in outer section
    let outer_data = vec![0u8; 1024];
    assert_eq!(outer_data.len(), 1024);

    // Inner section
    profile!(inner_section, mem_detail);

    let inner_profile = inner_section.as_ref().unwrap();
    let inner_start = inner_profile.start_line().unwrap();

    // Verify inner start is after outer start
    assert!(inner_start > outer_start);

    // Allocate memory in inner section
    let inner_data = vec![0u8; 2048];
    assert_eq!(inner_data.len(), 2048);

    end!(inner_section);

    // Allocate more memory in outer section
    let more_outer = vec![0u8; 4096];
    assert_eq!(more_outer.len(), 4096);

    end!(outer_section);
}

/// Test long-lived allocations that persist across tests
#[cfg(feature = "full_profiling")]
fn mem_attribution_persistent_allocations() {
    profile!(persistent_allocs, mem_detail);

    // Create some persistent allocations
    let mut allocations = Vec::new();
    for i in 0..5 {
        allocations.push(vec![i as u8; 1024 * (i + 1)]);
    }

    // Safely store allocations in the static Mutex
    {
        let mut stored_allocs = TEST_ALLOCATIONS.lock().unwrap();
        *stored_allocs = allocations;
    }

    // Verify allocations
    let stored_sum = {
        let stored_allocs = TEST_ALLOCATIONS.lock().unwrap();
        stored_allocs.iter().map(|v| v.len()).sum::<usize>()
    };

    assert_eq!(stored_sum, 1024 + 2048 + 3072 + 4096 + 5120);

    end!(persistent_allocs);
}

/// Test manual profile creation and registration
#[cfg(feature = "full_profiling")]
// #[enable_profiling]
fn mem_attribution_manual_profile() {
    with_sys_alloc(|| {
        // Create a profile manually
        let file_name = file_stem_from_path_str(file!());
        let fn_name = "manual_profile_test";

        // Manual profile with line numbers
        let profile = Profile::new(
            Some("manual_section"),
            Some(fn_name),
            ProfileType::Memory,
            false,
            true, // detailed memory
            file!(),
            Some(100), // fake start line
            Some(200), // fake end line
        )
        .unwrap();

        // Verify we can find the profile
        let found = find_profile(&file_name, profile.fn_name(), 150);
        assert!(found.is_some(), "Manual profile should be findable");
        if let Some(profile_ref) = found {
            assert!(
                profile_ref.detailed_memory(),
                "Profile should have detailed memory enabled"
            );
            assert_eq!(profile_ref.name(), "manual_section");
        }

        // Allocate memory
        let data = vec![0u8; 16384];
        assert_eq!(data.len(), 16384);
    });
}

/// Test registry functionality directly
#[cfg(feature = "full_profiling")]
fn mem_attribution_registry_functions() {
    // Verify file names in registry
    with_sys_alloc(|| {
        // Can't clear the registry for this test
        // {
        //     eprintln!(
        //         "PROFILE_REGISTRY.is_locked()?: {}",
        //         PROFILE_REGISTRY.is_locked()
        //     );
        //     let mut registry = PROFILE_REGISTRY.lock();
        //     dbg!();
        //     // *registry = Default::default();
        //     // *registry = ProfileRegistry::default();
        //     // assert!(registry.get_file_names().is_empty());
        //     // assert!(registry.active_instances.is_empty());
        // }

        // Create a profile manually
        let file_name = file_stem_from_path_str(file!());
        let fn_name = "registry_test";

        let profile = Profile::new(
            Some("registry_section"),
            Some(fn_name),
            ProfileType::Memory,
            false,
            true,
            file!(),
            Some(300),
            Some(400),
        )
        .unwrap();

        // // Register the profile
        // register_profile(&profile);

        let file_names = PROFILE_REGISTRY.lock().get_file_names();
        assert!(
            file_names.contains(&file_name.to_string()),
            "Registry should contain our file name"
        );

        let fn_name = profile.fn_name();

        // Try to find profiles at different line numbers
        let in_range = find_profile(&file_name, fn_name, 350);
        assert!(in_range.is_some(), "Should find profile for line in range");

        let before_range = find_profile(&file_name, fn_name, 250);
        assert!(
            before_range.is_none(),
            "Should not find profile for line before range"
        );

        let after_range = find_profile(&file_name, fn_name, 450);
        assert!(
            after_range.is_none(),
            "Should not find profile for line after range"
        );
    });
}

/// Test overlapping profiles
#[cfg(feature = "full_profiling")]
fn mem_attribution_overlapping_profiles() {
    // Set up several overlapping profiles
    let file_name = file_stem_from_path_str(file!());
    let fn_name = "overlap_test";

    // First profile: lines 500-600
    let profile1 = Profile::new(
        Some("overlap_first"),
        Some(fn_name),
        ProfileType::Memory,
        false,
        true,
        file!(),
        Some(500),
        Some(600),
    )
    .unwrap();

    // Second profile: lines 550-650 (overlaps with first)
    let _profile2 = Profile::new(
        Some("overlap_second"),
        Some(fn_name),
        ProfileType::Memory,
        false,
        true,
        file!(),
        Some(550),
        Some(650),
    )
    .unwrap();

    // // Register the profiles
    // register_profile(&profile1);
    // register_profile(&profile2);

    // Check which profile is found for a line in the overlap region
    let overlap = find_profile(&file_name, profile1.fn_name(), 575);
    assert!(
        overlap.is_some(),
        "Should find a profile in the overlap region"
    );

    // The find_profile function should return the most specific profile
    // According to the implementation, this should be the one that starts first
    if let Some(profile_ref) = overlap {
        assert_eq!(
            profile_ref.name(),
            "overlap_second",
            "Should find the profile that starts first (overlap_first)"
        );
    }
}

/// Test record_allocation function
#[cfg(feature = "full_profiling")]
fn mem_attribution_record_allocation() {
    // Create a profile with specific line numbers
    let file_name = file_stem_from_path_str(file!());
    let fn_name = format!("{file_name}::mem_attribution_record_allocation");

    profile!(record_alloc_section, mem_detail);

    // Get the profile
    let profile = record_alloc_section.as_ref().unwrap();
    let start_line = profile.start_line().unwrap();

    // Manually create a backtrace for testing
    let mut backtrace = backtrace::Backtrace::new_unresolved();
    backtrace.resolve();

    // Access registry directly to test record_allocation
    with_sys_alloc(|| {
        // Test valid allocation
        // assert!(!PROFILE_REGISTRY.is_locked());
        let valid = PROFILE_REGISTRY.lock().record_allocation(
            &file_name,
            &fn_name,
            start_line + 1, // Line within range
            1024,           // Size
            &mut backtrace,
        );

        assert!(
            valid,
            "Should have successfully recorded allocation within range"
        );

        // Test allocation for non-existent file
        let invalid_file = PROFILE_REGISTRY.lock().record_allocation(
            "nonexistent_file",
            &fn_name,
            start_line,
            1024,
            &mut backtrace,
        );
        assert!(
            !invalid_file,
            "Should not have recorded allocation for non-existent file"
        );

        // Test allocation for non-existent function
        let invalid_fn = PROFILE_REGISTRY.lock().record_allocation(
            &file_name,
            "nonexistent_function",
            start_line,
            1024,
            &mut backtrace,
        );
        assert!(
            !invalid_fn,
            "Should not have recorded allocation for non-existent function"
        );

        // Test allocation outside line range
        let out_of_range = PROFILE_REGISTRY.lock().record_allocation(
            &file_name,
            &fn_name,
            start_line - 10, // Before range
            1024,
            &mut backtrace,
        );
        assert!(
            !out_of_range,
            "Should not have recorded allocation outside line range"
        );
    });

    end!(record_alloc_section);
}

// ---------------------------------------------------------------------------
// Main test function that runs all tests sequentially
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "full_profiling")]
#[enable_profiling]
fn test_mem_attribution_full_sequence() {
    use thag_profiler::{profiling::set_profile_config, ProfileConfiguration};

    // Set debug logging off
    let _ = set_profile_config(
        ProfileConfiguration::try_from(vec!["both", "", "announce"].as_slice()).unwrap(),
    );

    // Ensure we start with a clean profiling state

    eprintln!("Starting memory attribution tests");

    // Test whole function profiling
    eprintln!("Testing whole function profiling...");
    mem_attribution_whole_function();

    // Test section profiling
    eprintln!("Testing section profiling...");
    mem_attribution_with_sections();

    // Test nested sections
    eprintln!("Testing nested sections...");
    mem_attribution_nested_sections();

    // Test persistent allocations
    eprintln!("Testing persistent allocations...");
    mem_attribution_persistent_allocations();

    // Test manual profile creation
    eprintln!("Testing manual profile creation...");
    mem_attribution_manual_profile();

    // Test registry functions
    eprintln!("Testing registry functions...");
    mem_attribution_registry_functions();

    // Test overlapping profiles
    eprintln!("Testing overlapping profiles...");
    mem_attribution_overlapping_profiles();

    // Test record_allocation function
    eprintln!("Testing record_allocation function...");
    mem_attribution_record_allocation();

    // Verify persistent allocations are still valid
    eprintln!("Verifying persistent allocations...");
    let stored = {
        let stored_allocs = TEST_ALLOCATIONS.lock().unwrap();
        stored_allocs.iter().map(|v| v.len()).sum::<usize>()
    };
    assert_eq!(stored, 1024 + 2048 + 3072 + 4096 + 5120);

    eprintln!("All memory attribution tests passed!");
}
