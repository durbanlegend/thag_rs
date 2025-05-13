/// This file contains examples of how to use the warn_once abstraction
/// to replace the custom warning pattern in record_dealloc.

#[cfg(test)]
mod tests {
    use thag_profiler::{debug_log, profiling::ProfileType, warn_once, warn_once_with_id};

    #[cfg(feature = "full_profiling")]
    use thag_profiler::mem_tracking::with_sys_alloc;

    /// Example of using the warn_once! macro in a function similar to record_dealloc
    #[test]
    #[cfg(feature = "full_profiling")]
    fn test_example_function_with_warn_once() {
        // Use the system allocator
        with_sys_alloc(|| {
            // Set profile type to trigger warning
            thag_profiler::profiling::set_global_profile_type(ProfileType::Time);

            // Call example function multiple times
            example_function_with_warn_once();
            example_function_with_warn_once();
            example_function_with_warn_once();

            // Set profile type to allow processing
            thag_profiler::profiling::set_global_profile_type(ProfileType::Memory);

            // Call example function again - this time it should execute fully
            example_function_with_warn_once();
        });
    }

    /// Example of how record_dealloc could be refactored to use warn_once!
    fn example_function_with_warn_once() {
        debug_log!("Entering example function");

        // Get profile type for condition check
        let profile_type = thag_profiler::get_global_profile_type();
        let is_mem_prof = profile_type == ProfileType::Memory || profile_type == ProfileType::Both;

        // Use the warn_once! macro for clean, optimized warning suppression
        warn_once!(
            !is_mem_prof,
            || {
                debug_log!(
                    "Example: Skipping processing because profile_type={:?}",
                    profile_type
                );
            },
            return
        );

        // Rest of the function logic that only runs when enabled
        debug_log!("Example: Running actual processing code");
    }

    /// Example of how record_dealloc could be refactored using warn_once_with_id
    #[test]
    #[cfg(feature = "full_profiling")]
    fn test_example_function_with_warn_once_id() {
        // Use the system allocator
        with_sys_alloc(|| {
            // Set profile type to trigger warning
            thag_profiler::profiling::set_global_profile_type(ProfileType::Time);

            // Call example function multiple times
            example_function_with_warn_once_id();
            example_function_with_warn_once_id();
            example_function_with_warn_once_id();

            // Set profile type to allow processing
            thag_profiler::profiling::set_global_profile_type(ProfileType::Memory);

            // Call example function again - this time it should execute fully
            example_function_with_warn_once_id();
        });
    }

    /// Example using the warn_once_with_id function
    fn example_function_with_warn_once_id() {
        debug_log!("Entering example function with ID");

        // Get profile type for condition check
        let profile_type = thag_profiler::get_global_profile_type();
        let is_mem_prof = profile_type == ProfileType::Memory || profile_type == ProfileType::Both;

        // Unique ID for this function (could be based on line number or hand-assigned)
        const WARNING_ID: usize = 42;

        // Using warn_once_with_id for warning suppression
        unsafe {
            // Using a unique ID for this specific warning
            if warn_once_with_id(WARNING_ID, !is_mem_prof, || {
                debug_log!(
                    "Example with ID: Skipping processing because profile_type={:?}",
                    profile_type
                );
            }) {
                return;
            }
        }

        // Rest of the function logic that only runs when enabled
        debug_log!("Example with ID: Running actual processing code");
    }

    /// Example of how to apply the pattern to record_dealloc
    #[test]
    fn test_refactored_record_dealloc() {
        // This is a placeholder showing how record_dealloc would be refactored
        debug_log!("This is just an example - the actual implementation would need to be done in mem_tracking.rs");

        /*
        // The record_dealloc function would be refactored like this:
        fn record_dealloc(address: usize, size: usize) {
            // Simple recursion prevention without using TLS with destructors
            static mut IN_TRACKING: bool = false;
            struct Guard;
            impl Drop for Guard {
                fn drop(&mut self) {
                    unsafe {
                        IN_TRACKING = false;
                    }
                }
            }

            // Flag if we're already tracking to prevent recursion
            let in_tracking = unsafe { IN_TRACKING };
            if in_tracking {
                debug_log!("*** Caution: already tracking: proceeding for deallocation of {size} B");
            }

            // Set tracking flag and create guard for cleanup
            unsafe { IN_TRACKING = true; }
            let _guard = Guard;

            let root_module = lazy_static_var(str, get_root_module().unwrap_or("root module"));
            let is_mem_prof = lazy_static_var(
                bool,
                get_global_profile_type() == ProfileType::Memory || profile_type == ProfileType::Both,
            );

            // Use the warn_once! macro for clean, optimized warning suppression
            warn_once!(!is_mem_prof, || {
                debug_log!(
                    "Skipping deallocation recording because profile_type={:?}",
                    profile_type
                );
            }, return);

            // ... rest of the record_dealloc implementation ...
        }
        */
    }
}
