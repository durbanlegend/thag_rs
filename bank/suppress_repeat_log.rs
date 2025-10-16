macro_rules! lazy_static_var {
    ($type:ty, deref, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}

pub fn foo() {
    // Fast path using non-atomic bool for zero overhead after first warning
    static mut WARNED: bool = false;
    // Thread-safe initialization using atomic
    static WARNED_ABOUT_SKIPPING: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    let is_mem_prof = lazy_static_var!(bool, false);

    if !is_mem_prof {
        // Fast path check - no synchronization overhead after first warning
        if unsafe { WARNED } {
            eprintln!("Fast path");
            return;
        }

        // Slow path with proper synchronization - only hit by the first few threads
        if !WARNED_ABOUT_SKIPPING.swap(true, std::sync::atomic::Ordering::Relaxed) {
            eprintln!("Skipping");
            // Update fast path flag for future calls
            unsafe {
                WARNED = true;
            }
        }
        return;
    }
    eprintln!("Not skipping");
}

for i in 1..=10 { foo() }
