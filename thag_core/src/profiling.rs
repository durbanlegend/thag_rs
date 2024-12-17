#![allow(clippy::module_name_repetitions)]
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, SystemTime};
pub use thag_proc_macros::profile;

static FIRST_WRITE: AtomicBool = AtomicBool::new(true);

thread_local! {
    static PROFILE_STACK: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);
static START_TIME: AtomicU64 = AtomicU64::new(0);

pub fn enable_profiling(enabled: bool) {
    PROFILING_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled {
        FIRST_WRITE.store(true, Ordering::SeqCst);
        // Store start time when profiling is enabled
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        START_TIME.store(now, Ordering::SeqCst);
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
            PROFILE_STACK.with(|stack| {
                stack.borrow_mut().push(name);
            });
            Some(Instant::now())
        } else {
            None
        };

        Self { start, name }
    }

    fn get_parent_stack() -> String {
        PROFILE_STACK.with(|stack| {
            let stack = stack.borrow();
            stack
                .iter()
                .take(stack.len().saturating_sub(1))
                .copied()
                .collect::<Vec<_>>()
                .join(";")
        })
    }

    fn write_trace_event(&self, duration: std::time::Duration) {
        if !is_profiling_enabled() {
            return;
        }

        let micros = duration.as_micros();
        if micros == 0 {
            return;
        }

        let first_write = FIRST_WRITE.swap(false, Ordering::SeqCst);
        if let Ok(mut file) = File::options()
            .create(true)
            .write(true)
            .truncate(first_write)
            .append(!first_write)
            .open("thag-profile.folded")
        {
            let stack = Self::get_parent_stack();
            let entry = if stack.is_empty() {
                self.name.to_string()
            } else {
                format!("{};{}", stack, self.name)
            };

            // Write just the stack and duration
            writeln!(file, "{} {}", entry, micros).ok();
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
    () => {
        const NAME: &'static str = concat!(module_path!(), "::", stringify!(profile_method));
        let _profile = $crate::profiling::Profile::new(NAME);
    };
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new($name);
    };
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

#[derive(Default)]
pub struct ProfileStats {
    pub calls: HashMap<String, u64>,
    pub total_time: HashMap<String, u128>, // Change to u128 for microseconds
    pub async_boundaries: HashSet<String>,
    // Keep existing fields for backwards compatibility
    count: u64,
    duration_total: std::time::Duration,
    min_time: Option<std::time::Duration>,
    max_time: Option<std::time::Duration>,
}

impl ProfileStats {
    pub fn record(&mut self, func_name: &str, duration: std::time::Duration) {
        *self.calls.entry(func_name.to_string()).or_default() += 1;
        *self.total_time.entry(func_name.to_string()).or_default() += duration.as_micros();
    }

    pub fn mark_async(&mut self, func_name: &str) {
        self.async_boundaries.insert(func_name.to_string());
    }

    #[must_use]
    pub fn average(&self) -> Option<std::time::Duration> {
        if self.count > 0 {
            let count = u32::try_from(self.count).unwrap_or(u32::MAX);
            Some(self.duration_total / count)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_async_boundary(&self, func_name: &str) -> bool {
        self.async_boundaries.contains(func_name)
    }

    // Getter methods for private fields if needed
    #[must_use]
    pub fn count(&self) -> u64 {
        self.count
    }

    #[must_use]
    pub fn total_duration(&self) -> std::time::Duration {
        self.duration_total
    }

    #[must_use]
    pub fn min_time(&self) -> Option<std::time::Duration> {
        self.min_time
    }

    #[must_use]
    pub fn max_time(&self) -> Option<std::time::Duration> {
        self.max_time
    }
}
