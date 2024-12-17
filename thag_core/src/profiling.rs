#![allow(clippy::module_name_repetitions)]
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
pub use thag_proc_macros::profile;

static FIRST_WRITE: AtomicBool = AtomicBool::new(true);

thread_local! {
    static PROFILE_STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);

// Reset the first_write flag when profiling is enabled
pub fn enable_profiling(enabled: bool) {
    PROFILING_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        FIRST_WRITE.store(true, Ordering::SeqCst);
    }
}

pub fn is_profiling_enabled() -> bool {
    PROFILING_ENABLED.load(Ordering::SeqCst)
}

pub struct Profile {
    start: Option<Instant>,
    name: &'static str,
}

impl Profile {
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        let start = if is_profiling_enabled() {
            // println!("Creating profile for: {name}"); // Temporary debug output
            PROFILE_STACK.with(|stack| {
                stack.borrow_mut().push(name);
            });
            Some(Instant::now())
        } else {
            // println!("Profiling is disabled"); // Temporary debug output
            None
        };

        Self { start, name }
    }

    fn get_parent_stack() -> String {
        PROFILE_STACK.with(|stack| {
            let stack = stack.borrow();
            // Don't include the immediate parent in the child's stack
            stack
                .iter()
                .take(stack.len().saturating_sub(1))
                .rev() // Reverse to get root->leaf order
                .copied()
                .collect::<Vec<_>>()
                .join(";")
        })
    }

    fn write_trace_event(&self, duration: std::time::Duration) {
        if let Ok(mut file) = File::options()
            .create(true)
            .write(true)
            .truncate(FIRST_WRITE.swap(false, Ordering::SeqCst))
            .append(!FIRST_WRITE.load(Ordering::SeqCst))
            .open("thag-profile.folded")
        {
            // Only write if there's actual time spent in this function
            if duration.as_micros() > 0 {
                writeln!(
                    file,
                    "{};{} {}",
                    self.name,
                    Self::get_parent_stack(),
                    duration.as_micros()
                )
                .ok();
            }
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let elapsed = start.elapsed();
            self.write_trace_event(elapsed);
            PROFILE_STACK.with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }
}

// Macro for convenient usage
#[macro_export]
macro_rules! profile {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
}

/// Profiles a specific section of code
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
}

#[macro_export]
macro_rules! profile_method {
    () => {{
        const NAME: &'static str = concat!(module_path!(), "::", stringify!(profile_method));
        let _profile = $crate::profiling::Profile::new(NAME);
    }};
    ($name:expr) => {{
        let _profile = $crate::profiling::Profile::new($name);
    }};
}

// Optional: A more detailed version that includes file and line information
#[macro_export]
macro_rules! profile_method_detailed {
    () => {
        let _profile = $crate::profiling::Profile::new(concat!(
            module_path!(),
            "::",
            function_name!(),
            " at ",
            file!(),
            ":",
            line!()
        ));
    };
}

// Optional: Collection of timing statistics
#[derive(Default)]
pub struct ProfileStats {
    count: u64,
    total_time: std::time::Duration,
    min_time: Option<std::time::Duration>,
    max_time: Option<std::time::Duration>,
}

impl ProfileStats {
    pub fn record(&mut self, duration: std::time::Duration) {
        self.count += 1;
        self.total_time += duration;
        self.min_time = Some(self.min_time.map_or(duration, |min| min.min(duration)));
        self.max_time = Some(self.max_time.map_or(duration, |max| max.max(duration)));
    }

    #[must_use]
    pub fn average(&self) -> Option<std::time::Duration> {
        if self.count > 0 {
            // Convert count to u32 for division
            let count = u32::try_from(self.count).unwrap_or(u32::MAX);
            Some(self.total_time / count)
        } else {
            None
        }
    }
}
