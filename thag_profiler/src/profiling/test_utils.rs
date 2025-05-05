//! This module contains utilities for testing profiling functionality.
//! These are not part of the public API and are only used for internal tests.

use crate::profiling::{enable_profiling, ProfileType, PROFILING_STATE, TEST_MODE_ACTIVE};
use std::sync::atomic::Ordering;

/// Initializes profiling for tests
/// 
/// This is an internal function provided for tests to initialize profiling
/// without exposing the implementation details of how profiling is enabled.
/// 
/// # Arguments
/// * `profile_type` - The type of profiling to enable
pub fn initialize_profiling_for_test(profile_type: ProfileType) -> crate::ProfileResult<()> {
    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Then enable profiling using the internal function
    enable_profiling(true, Some(profile_type))
}

/// Safely cleans up profiling after a test
pub fn cleanup_profiling_after_test() -> crate::ProfileResult<()> {
    // First disable profiling
    let result = enable_profiling(false, None);

    // Reset test mode flag
    TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

    result
}

/// Force sets the profiling state for testing purposes
/// This is only used in tests to directly manipulate the profiling state
pub fn force_set_profiling_state(enabled: bool) {
    // This function is only used in tests to directly manipulate the profiling state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
}