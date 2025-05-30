#![allow(clippy::uninlined_format_args)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling. It also contains
//! the custom memory allocator implementation that enables memory profiling.

use crate::{
    debug_log, file_stem_from_path, find_profile, flush_debug_log, get_global_profile_type,
    get_root_module, is_detailed_memory, lazy_static_var,
    mem_attribution::{DetailedAddressRegistry, ProfileReg},
    profiling::{
        build_stack, clean_function_name, extract_alloc_callstack,
        extract_detailed_alloc_callstack, get_memory_detail_dealloc_path, get_memory_detail_path,
        get_memory_path, is_profiling_state_enabled, MemoryDetailDeallocFile, MemoryDetailFile,
        MemoryProfileFile,
    },
    regex, warn_once, Profile, ProfileRef, ProfileType,
};
use backtrace::Backtrace;
use parking_lot::Mutex;
use regex::Regex;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::{HashMap, HashSet},
    env, fmt,
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        LazyLock,
    },
    time::Instant,
};

// Fast path atomic for checking current allocator without locking
pub static ALLOC_START_PATTERN: LazyLock<&'static Regex> =
    LazyLock::new(|| regex!("thag_profiler::mem_tracking.+Dispatcher"));

// Static atomics for minimal state tracking without allocations
static USING_SYSTEM_ALLOCATOR: AtomicBool = AtomicBool::new(false);

// static RECURSION_DEPTH: AtomicUsize = AtomicUsize::new(0);

// Maximum safe allocation size - 1 GB, anything larger is suspicious
const MAX_SAFE_ALLOCATION: usize = 1024 * 1024 * 1024;

// Define allocator types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Task-aware allocator that tracks which task allocated memory
    Tracking,
    /// System allocator for profiling operations
    System,
}

impl fmt::Display for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tracking => write!(f, "Tracking"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Get the current allocator based on the atomic state
#[inline]
pub fn current_allocator() -> Allocator {
    if USING_SYSTEM_ALLOCATOR.load(Ordering::SeqCst) {
        // eprintln!("Using system allocator");
        Allocator::System
    } else {
        Allocator::Tracking
    }
}

/// Run a function with the system allocator
///
/// This function temporarily switches to the system allocator while executing the provided
/// closure, then switches back to the previous allocator afterward.
#[inline(always)]
pub fn with_sys_alloc<T>(f: impl FnOnce() -> T) -> T {
    // Set the flag directly
    let using_sys_alloc = USING_SYSTEM_ALLOCATOR.load(Ordering::SeqCst);
    if !using_sys_alloc {
        USING_SYSTEM_ALLOCATOR.store(true, Ordering::SeqCst);
    }

    // Execute the function
    let result = f();

    // Restore the flag (no guard needed)
    if !using_sys_alloc {
        USING_SYSTEM_ALLOCATOR.store(false, Ordering::SeqCst);
    }

    result
}

/// Dispatcher allocator that routes allocation requests to the appropriate allocator
pub struct Dispatcher {
    pub tracking: TrackingAllocator,
    pub system: std::alloc::System,
}

impl Dispatcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tracking: TrackingAllocator,
            system: std::alloc::System,
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let current = current_allocator();

        // // For debugging, log larger allocations
        // if layout.size() > 1024 * 1024 {
        //     // 1MB
        //     with_sys_alloc(|| {
        //         debug_log!(
        //             "Large allocation of {} bytes using allocator: {:?}",
        //             layout.size(),
        //             current
        //         )
        //     });
        // }

        match current {
            Allocator::System => unsafe { self.system.alloc(layout) },
            Allocator::Tracking => {
                // // Use a recursive guard here to prevent infinite loops
                // let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                // if recursion_depth > 10 {
                //     // Emergency fallback to system allocator
                //     unsafe { self.system.alloc(layout) }
                // } else {
                //     RECURSION_DEPTH.store(recursion_depth + 1, Ordering::SeqCst);
                //     let ptr = unsafe { self.tracking.alloc(layout) };
                //     let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                //     if recursion_depth > 0 {
                //         RECURSION_DEPTH.store(recursion_depth - 1, Ordering::SeqCst);
                //     }
                //     ptr
                // }
                unsafe { self.tracking.alloc(layout) }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        // Safety check for unreasonably large deallocations
        if layout.size() > MAX_SAFE_ALLOCATION {
            with_sys_alloc(|| {
                eprintln!(
                    "WARNING: Extremely large deallocation request of {} bytes",
                    layout.size()
                );
            });
            // Still need to deallocate it to avoid memory leaks
        }

        match current_allocator() {
            Allocator::System => unsafe { self.system.dealloc(ptr, layout) },
            Allocator::Tracking => {
                // // Use a recursive guard here to prevent infinite loops
                // let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                // if recursion_depth > 10 {
                //     // Emergency fallback to system allocator
                //     unsafe { self.system.dealloc(ptr, layout) }
                // } else {
                //     RECURSION_DEPTH.store(recursion_depth + 1, Ordering::SeqCst);
                //     unsafe { self.tracking.dealloc(ptr, layout) };
                //     let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                //     if recursion_depth > 0 {
                //         RECURSION_DEPTH.store(recursion_depth - 1, Ordering::SeqCst);
                //     }
                // }
                unsafe { self.tracking.dealloc(ptr, layout) }
            }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if ptr.is_null() {
            return unsafe {
                self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()))
            };
        }

        // Safety check for unreasonably large reallocations
        // if new_size > MAX_SAFE_ALLOCATION {
        //     with_sys_alloc(|| {
        //         eprintln!(
        //             "WARNING: Extremely large reallocation request of {} bytes",
        //             layout.size()
        //         )
        //     });
        //     return std::ptr::null_mut();
        // }

        match current_allocator() {
            Allocator::System => unsafe { self.system.realloc(ptr, layout, new_size) },
            Allocator::Tracking => {
                // // Use a recursive guard here to prevent infinite loops
                // let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                // if recursion_depth > 10 {
                //     // Emergency fallback to system allocator
                //     unsafe { self.system.realloc(ptr, layout, new_size) }
                // } else {
                //     RECURSION_DEPTH.store(recursion_depth + 1, Ordering::SeqCst);
                //     let ptr = unsafe { self.tracking.realloc(ptr, layout, new_size) };
                //     let recursion_depth = RECURSION_DEPTH.load(Ordering::Relaxed);
                //     if recursion_depth > 0 {
                //         RECURSION_DEPTH.store(recursion_depth - 1, Ordering::SeqCst);
                //     }
                //     ptr
                // }
                unsafe { self.tracking.realloc(ptr, layout, new_size) }
            }
        }
    }
}

/// Task-aware allocator that tracks memory allocations
pub struct TrackingAllocator;

// Static instance for global access
static TRACKING_ALLOCATOR: TrackingAllocator = TrackingAllocator;

// Helper to get the allocator instance
#[must_use]
pub fn get_allocator() -> &'static TrackingAllocator {
    &TRACKING_ALLOCATOR
}

#[allow(clippy::unused_self)]
impl TrackingAllocator {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };

        with_sys_alloc(|| {
            if !ptr.is_null() && is_profiling_state_enabled() {
                let size = layout.size();
                // Potentially skip small allocations
                if size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_alloc(address, size);
                }
            }
            // See ya later allocator
            ptr
        })
    }

    #[allow(clippy::too_many_lines)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        with_sys_alloc(|| {
            if !ptr.is_null()
                && is_profiling_state_enabled()
                // Only record detailed deallocations to -memory_detail_dealloc.folded if requested
                && lazy_static_var!(bool, deref, is_detailed_memory())
            {
                // Potentially skip small allocations
                let size = layout.size();
                if size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_dealloc(address, size);
                }
            }
        });

        // Forward to system allocator for deallocation
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        with_sys_alloc(|| {
            if !ptr.is_null()
                && is_profiling_state_enabled()
                // Only record detailed deallocations to -memory_detail_dealloc.folded if requested
                && lazy_static_var!(bool, deref, is_detailed_memory())
            {
                // Potentially skip small allocations
                let dealloc_size = layout.size();
                if dealloc_size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_dealloc(address, dealloc_size);
                }
            }

            // Potentially skip small allocations
            if new_size > *SIZE_TRACKING_THRESHOLD {
                let address = ptr as usize;
                record_alloc(address, new_size);
            }
        });

        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[allow(clippy::too_many_lines, unreachable_code, unused_variables)]
fn record_alloc(address: usize, size: usize) {
    // static TOTAL_BYTES: AtomicUsize = AtomicUsize::new(0);
    // TOTAL_BYTES.fetch_add(size, Ordering::Relaxed);

    // return;

    // unreachable!();

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

    // assert_eq!(current_allocator(), Allocator::System);

    with_sys_alloc(|| {
        if size == 0 {
            debug_log!("Zero-sized allocation found");
            return;
        }

        let profile_type = get_global_profile_type();
        if profile_type != ProfileType::Memory && profile_type != ProfileType::Both {
            // debug_log!(
            //     "Skipping allocation recording because profile_type={:?}",
            //     profile_type
            // );
            return;
        }

        // Flag if we're already tracking in case it causes an infinite recursion
        let in_tracking = unsafe { IN_TRACKING };

        // Assertion disabled because not 100%
        // #[cfg(debug_assertions)]
        // assert!(!in_tracking);

        if in_tracking {
            debug_log!("*** Caution: already tracking: proceeding for deallocation of {size} B");
            // return ptr;
        }

        // Set tracking flag and create guard for cleanup
        unsafe {
            IN_TRACKING = true;
        }
        let _guard = Guard;

        // Get backtrace without recursion
        // debug_log!("Attempting backtrace");
        let start_ident = Instant::now();
        // Now we can safely use backtrace without recursion!
        // debug_log!("Calling extract_callstack");
        let mut current_backtrace = with_sys_alloc(|| Backtrace::new_unresolved());

        // TODO phase out - useful for debugging though
        // let cleaned_stack = extract_alloc_callstack(&ALLOC_START_PATTERN, &mut current_backtrace);
        // debug_log!("Cleaned_stack for size={size}: {cleaned_stack:?}");
        // let in_profile_code = cleaned_stack
        //     .iter()
        //     .any(|frame| frame.contains("Backtrace::new") || frame.contains("Profile::new"));

        // if in_profile_code {
        //     debug_log!("Ignoring allocation request of size {size} for profiler code");
        //     return;
        // }

        let file_names = {
            with_sys_alloc(|| {
                ProfileReg::get()
                    // .lock()
                    .get_file_names()
            })
        };
        debug_log!("file_names={file_names:#?}");

        // let Some((filename, lineno, frame, fn_name, profile_ref)) = Backtrace::frames(&current_backtrace)
        let Some(frames) =
            extract_callstack_with_recursion_check(&mut current_backtrace, file_names)
        else {
            eprintln!("*** Recursion detected ***");
            return;
        };

        with_sys_alloc(|| {
            if frames.is_empty() {
                debug_log!("No eligible profile found");
                return;
            }
            // debug_log!("func_and_ancestors={func_and_ancestors:#?}");

            let in_profile_code = frames.iter().any(|(_, _, frame, _, _)| {
                frame.contains("Backtrace::new") || frame.contains("Profile::new")
            });

            if in_profile_code {
                debug_log!("Ignoring allocation request of size {size} for profiler code");
                return;
            }

            let (filename, lineno, frame, fn_name, _profile_ref) = &frames[0];

            debug_log!("Found filename (file_name)={filename}, lineno={lineno}, fn_name: {fn_name:?}, frame: {frame:?}");

            // Still record detailed allocations to -memory_detail.folded if requested
            let detailed_memory = lazy_static_var!(bool, deref, is_detailed_memory());
            if detailed_memory {
                record_detailed_alloc(
                    address,
                    size,
                    &ALLOC_START_PATTERN,
                    &mut current_backtrace,
                    true,
                );
            }

            // Try to record the allocation in the new profile registry
            if !filename.is_empty()
                && *lineno > 0
                && record_allocation(filename, fn_name, *lineno, size, &mut current_backtrace)
            {
                debug_log!("Recorded allocation of {size} bytes in {filename}::{fn_name}:{lineno} to a profile");

                debug_log!(
                    "size={size}, time to assign = {}ms",
                    start_ident.elapsed().as_millis()
                );
            }
        });
    });
}

// Don't change name from "extract_callstack_..." as this is used in regression checking.
fn extract_callstack_with_recursion_check(
    current_backtrace: &mut Backtrace,
    file_names: Vec<String>,
) -> Option<Vec<(String, u32, String, String, ProfileRef)>> {
    with_sys_alloc(|| current_backtrace.resolve());
    with_sys_alloc(|| {
        // Pre-allocate with fixed capacity to avoid reallocations
        let mut frames: Vec<(String, u32, String, String, ProfileRef)> = Vec::with_capacity(100); // Fixed size, no growing
        let mut found_recursion = false;
        let mut stack_size: usize = with_sys_alloc(|| 0);
        for (i, frame) in Backtrace::frames(current_backtrace).iter().enumerate() {
            if i >= 200 {
                with_sys_alloc(|| {
                    println!("current_backtrace={current_backtrace:#?}");
                });
                panic!("Max limit of 200 frames exceeded");
            } // Hard limit to prevent overflow
            for symbol in frame.symbols() {
                let maybe_frame = with_sys_alloc(|| symbol.name());
                if let Some(ref name) = &maybe_frame {
                    let name_str = with_sys_alloc(|| format!("{name}"));

                    // Check for our own functions (recursion detection)
                    if name_str.contains("extract_callstack_with_recursion_check") {
                        // with_sys_alloc(|| {
                        //     eprintln!("current_backtrace={current_backtrace:#?}");
                        // });
                        found_recursion = true;
                        break;
                    }
                }

                let maybe_filename = symbol.filename();
                let maybe_lineno = symbol.lineno();

                // Apply the first filter
                if maybe_filename.is_none()
                    || maybe_lineno.is_none()
                    || maybe_frame.is_none()
                    || with_sys_alloc(|| maybe_frame.as_ref().unwrap().to_string()).starts_with('<')
                {
                    continue;
                }

                // Safe to unwrap now
                let filename = with_sys_alloc(|| file_stem_from_path(maybe_filename.unwrap()));
                let lineno = with_sys_alloc(|| maybe_lineno.unwrap());
                let symbol_name = with_sys_alloc(|| maybe_frame.unwrap());
                let frame_str = with_sys_alloc(|| symbol_name.to_string());

                // Apply second filter
                if !file_names.contains(&filename) {
                    continue;
                }

                let mut frame = with_sys_alloc(|| frame_str.clone());
                let fn_name = with_sys_alloc(|| clean_function_name(&mut frame));

                let maybe_profile_ref = find_profile(&filename, &fn_name, lineno);
                if let Some(profile_ref) = maybe_profile_ref {
                    // Safe to add this frame
                    stack_size += 1;
                    // Hard limit to prevent overflow
                    if stack_size > 100 {
                        with_sys_alloc(|| {
                            println!("current_backtrace={current_backtrace:#?}");
                        });
                        panic!("Max limit of 200 frames exceeded");
                    }
                    // stack_size
                    frames.push((filename, lineno, frame_str, fn_name, profile_ref));
                }
            }
            if found_recursion {
                break; // Exit early if recursion detected
            }
        }
        if found_recursion {
            None // Signal to skip tracking
        } else {
            Some(frames)
        }
    })
}

/// Record an allocation with the profile registry based on module path and line number
pub fn record_allocation(
    file_name: &str,
    fn_name: &str,
    line: u32,
    size: usize,
    current_backtrace: &mut Backtrace,
) -> bool {
    with_sys_alloc(|| {
        // First log (acquires debug log mutex)
        debug_log!(
            "Looking for profile to record allocation: module={file_name}, fn={fn_name}, line={line}, size={size}"
        );

        // Flush to release the debug log mutex
        flush_debug_log();

        // Print list of registered modules to help diagnose issues
        {
            let modules = ProfileReg::get()
                // .lock()
                .get_file_names();
            debug_log!("Available modules in registry: {modules:?}");
            flush_debug_log();
        }

        // Now acquire the PROFILE_REGISTRY mutex
        let result;
        {
            debug_log!("About to call record_allocation on registry");
            // result = crate::mem_attribution::ProfileReg::get()
            result = ProfileReg::get().record_allocation(
                file_name,
                fn_name,
                line,
                size,
                current_backtrace,
            );
            debug_log!("record_allocation on registry returned {result}");
        }

        // Log after releasing the mutex
        if result {
            debug_log!(
                "Successfully recorded allocation of {size} bytes in module {file_name}::{fn_name} at line {line}"
            );
        } else {
            debug_log!("No matching profile found to record allocation of {size} bytes in module {file_name}::{fn_name} at line {line}");
        }
        // flush_debug_log();

        result
    })
}

pub fn register_detailed_allocation(address: usize, size: usize, stack: Vec<String>) {
    if is_detailed_memory() {
        DetailedAddressRegistry::get().insert(address, (stack, size));
    }
}

pub fn record_detailed_alloc(
    address: usize,
    size: usize,
    start_pattern: &Regex,
    current_backtrace: &mut Backtrace,
    write_to_detail_file: bool,
) {
    let detailed_stack = extract_detailed_alloc_callstack(start_pattern, current_backtrace);
    write_detailed_stack_alloc(size, write_to_detail_file, &detailed_stack);
    register_detailed_allocation(address, size, detailed_stack);
}

#[allow(
    clippy::ptr_arg,
    clippy::missing_panics_doc,
    reason = "debug_assertions"
)]
pub fn write_detailed_stack_alloc(
    size: usize,
    write_to_detail_file: bool,
    detailed_stack: &Vec<String>,
) {
    // #[cfg(debug_assertions)]
    // assert!(!detailed_stack.is_empty());
    let root_module = lazy_static_var!(
        String,
        get_root_module()
            .as_ref()
            .map_or("root module", |v| v)
            .to_string()
    );

    let entry = if detailed_stack.is_empty() {
        format!("[Out of `{root_module}` scope] {size}")
    } else {
        let descr_stack = build_stack(detailed_stack, None, ";");

        debug_log!("descr_stack={descr_stack}");
        format!("{descr_stack} {size}")
    };

    let (memory_path, file) = if write_to_detail_file {
        (get_memory_detail_path().unwrap(), MemoryDetailFile::get())
    } else {
        (get_memory_path().unwrap(), MemoryProfileFile::get())
    };
    let _ = Profile::write_profile_event(memory_path, file, &entry);
}

#[allow(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    reason = "debug_assertions"
)]
pub fn record_dealloc(address: usize, size: usize) {
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

    // Assertion disabled because not 100%
    // #[cfg(debug_assertions)]
    // assert_eq!(current_allocator(), Allocator::System);

    let root_module = lazy_static_var!(
        String,
        get_root_module()
            .as_ref()
            .map_or("root module", |v| v)
            .to_string()
    );

    let profile_type = lazy_static_var!(ProfileType, deref, get_global_profile_type());
    let is_mem_prof = lazy_static_var!(bool, {
        profile_type == ProfileType::Memory || profile_type == ProfileType::Both
    });

    // Use the warn_once! macro for clean, optimized warning suppression
    warn_once!(
        !is_mem_prof,
        || {
            debug_log!("Skipping deallocation recording because profile_type={profile_type:?}");
        },
        return
    );

    // Flag if we're already tracking in case it causes an infinite recursion
    let in_tracking = unsafe { IN_TRACKING };

    // Assertion disabled because not 100%
    // #[cfg(debug_assertions)]
    // assert!(!in_tracking);

    if in_tracking {
        debug_log!("*** Caution: already tracking: proceeding for deallocation of {size} B");
        // return ptr;
    }

    // Set tracking flag and create guard for cleanup
    unsafe {
        IN_TRACKING = true;
    }
    let _guard = Guard;

    // Get backtrace without recursion
    // debug_log!("Attempting backtrace");
    // let start_ident = Instant::now();
    // let mut task_id = 0;
    // Now we can safely use backtrace without recursion!
    let start_pattern: &Regex = regex!("thag_profiler::mem_tracking.+Dispatcher");

    // debug_log!("Calling extract_callstack");
    let mut current_backtrace = Backtrace::new_unresolved();
    let cleaned_stack = extract_alloc_callstack(start_pattern, &mut current_backtrace);
    // debug_log!("Cleaned_stack for size={size}: {cleaned_stack:?}");
    let in_profile_code = cleaned_stack
        .iter()
        .any(|frame| frame.contains("::profiling::Profile"));

    if in_profile_code {
        debug_log!(
            "Summary memory tracking ignoring deallocation request of size {size} for profiler code: frame={:?}",
            cleaned_stack
                .iter()
                .find(|frame| frame.contains("::profiling::Profile"))
        );
        // debug_log!("...current backtrace: {:#?}", current_backtrace);
        return;
    }

    let detailed_memory = lazy_static_var!(bool, deref, is_detailed_memory());
    if size > 0 && detailed_memory {
        let detailed_stack =
            extract_detailed_alloc_callstack(start_pattern, &mut current_backtrace);

        let in_profile_code = detailed_stack
            .iter()
            .any(|frame| frame.contains("::profiling::Profile"));

        if in_profile_code {
            debug_log!(
                "Detailed memory tracking ignoring detailed deallocation request of size {size} for profiler code: frame={:?}",
                detailed_stack
                    .iter()
                    .find(|frame| frame.contains("::profiling::Profile"))
            );
            // debug_log!("...current backtrace: {:#?}", current_backtrace);
            return;
        }

        let entry = if detailed_stack.is_empty() {
            let stack_and_size = {
                DetailedAddressRegistry::get()
                    .remove(&address)
                    .unwrap_or((0, (Vec::new(), size)))
            };

            let (stack, _) = stack_and_size.1;

            let legend = if stack.is_empty() {
                // debug_log!("Empty cleaned_stack and stack for backtrace={current_backtrace:#?}");
                format!("[Dealloc out of `{root_module}` scope]")
            } else {
                stack.join(";")
            };
            format!("{legend} {size}")
        } else {
            format!("{} {size}", detailed_stack.join(";"))
        };

        let memory_detail_dealloc_path = get_memory_detail_dealloc_path().unwrap();
        let _ = Profile::write_profile_event(
            memory_detail_dealloc_path,
            MemoryDetailDeallocFile::get(),
            &entry,
        );
    }
}

// // Create a direct static instance
#[global_allocator]
static ALLOCATOR: Dispatcher = Dispatcher::new();

// ========== ALLOCATION TRACKING DEFINITIONS ==========

pub static SIZE_TRACKING_THRESHOLD: LazyLock<usize> = LazyLock::new(|| {
    let threshold = env::var("SIZE_TRACKING_THRESHOLD")
        .or_else(|_| Ok::<String, &str>(String::from("0")))
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .expect("Value specified for SIZE_TRACKING_THRESHOLD must be a valid integer");
    if threshold == 0 {
        debug_log!("*** The SIZE_TRACKING_THRESHOLD environment variable is set or defaulted to 0, so all memory allocations and deallocations will be tracked.");
    } else {
        debug_log!("*** Only memory allocations and deallocations exceeding the specified threshold of {threshold} bytes will be tracked.");
    }
    threshold
});

// /// Registry for tracking active profiles and task stacks
// #[derive(Debug)]
// struct ProfileRegistry {
//     /// Set of active task IDs
//     active_profiles: BTreeSet<usize>,
// }

// impl ProfileRegistry {
//     const fn new() -> Self {
//         Self {
//             active_profiles: BTreeSet::new(),
//         }
//     }

//     /// Add a task to active profiles
//     pub fn activate_task(&mut self, task_id: usize) {
//         self.active_profiles.insert(task_id);
//     }

//     /// Remove a task from active profiles
//     pub fn deactivate_task(&mut self, task_id: usize) {
//         self.active_profiles.remove(&task_id);
//     }

//     /// Get a copy of the active task IDs
//     pub fn get_active_tasks(&self) -> Vec<usize> {
//         self.active_profiles.iter().copied().collect()
//     }

//     /// Get the last (most recently added) active task
//     pub fn get_last_active_task(&self) -> Option<usize> {
//         self.active_profiles.iter().next_back().copied()
//     }
// }

// Global profile registry
// static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
//     LazyLock::new(|| Mutex::new(ProfileRegistry::new()));

// ========== PUBLIC REGISTRY API ==========

/// Add a task to active profiles
pub fn activate_task(task_id: usize) {
    with_sys_alloc(|| {
        ProfileReg::get().activate_task(task_id);
    });
}

/// Remove a task from active profiles
#[allow(dead_code)]
pub fn deactivate_task(task_id: usize) {
    with_sys_alloc(|| {
        ProfileReg::get().deactivate_task(task_id);
    });
}

// /// Add a task to a thread's stack
// pub fn push_task_to_stack(thread_id: ThreadId, task_id: usize) {
//     with_sys_alloc(|| {
//         PROFILE_REGISTRY
//             .lock()
//             .push_task_to_stack(thread_id, task_id);
//     });
// }

// /// Remove a task from a thread's stack
// #[allow(dead_code)]
// pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
//     with_sys_alloc(|| {
//         PROFILE_REGISTRY
//             .lock()
//             .pop_task_from_stack(thread_id, task_id);
//     });
// }

/// Get active tasks
#[must_use]
pub fn get_active_tasks() -> Vec<usize> {
    with_sys_alloc(|| ProfileReg::get().get_active_tasks())
}

/// Get the last active task
#[must_use]
pub fn get_last_active_task() -> Option<usize> {
    with_sys_alloc(|| ProfileReg::get().get_last_active_task())
}

// ========== TASK CONTEXT DEFINITIONS ==========

/// Task context for tracking allocations
#[derive(Debug, Clone)]
pub struct TaskMemoryContext {
    pub task_id: usize,
}

impl TaskMemoryContext {
    /// Gets the unique ID for this task
    #[must_use]
    pub const fn id(&self) -> usize {
        self.task_id
    }
}

// Provide a dummy TaskMemoryContext type for when full_profiling is disabled
#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskMemoryContext;

/// Creates a new task context for memory tracking.
#[must_use]
pub fn create_memory_task() -> TaskMemoryContext {
    let allocator = get_allocator();
    allocator.create_task_context()
}

pub fn trim_backtrace(start_pattern: &Regex, current_backtrace: &Backtrace) -> Vec<String> {
    Backtrace::frames(current_backtrace)
        .iter()
        .flat_map(backtrace::BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .skip_while(|element| !start_pattern.is_match(element))
        .take_while(|name| !name.contains("__rust_begin_short_backtrace"))
        .map(|mut name| clean_function_name(&mut name))
        .collect::<Vec<String>>()
}

// ========== TASK STATE MANAGEMENT ==========

// Task tracking state
pub struct TaskState {
    // Counter for generating task IDs
    pub next_task_id: AtomicUsize,
}

// Global task state
pub static TASK_STATE: LazyLock<TaskState> = LazyLock::new(|| TaskState {
    next_task_id: AtomicUsize::new(1),
});

// To handle active task tracking, instead of thread-locals, we'll use task-specific techniques
#[derive(Clone, Debug)]
pub struct TaskGuard {
    task_id: usize,
}

impl TaskGuard {
    #[must_use]
    pub const fn new(task_id: usize) -> Self {
        Self { task_id }
    }
}

#[cfg(not(feature = "full_profiling"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct TaskGuard;

impl Drop for TaskGuard {
    fn drop(&mut self) {
        // Run these operations with System allocator
        with_sys_alloc(|| {
            // Remove from active profiles
            debug_log!("Deactivating task {}", self.task_id);
            ProfileReg::get().deactivate_task(self.task_id);

            // Flush logs directly
            if let Some(logger) = crate::DebugLogger::get() {
                let _ = logger.lock().flush();
            }
        });
    }
}

// ========== TASK PATH MANAGEMENT ==========

// Task Path Registry for debugging
// 1. Declare the TASK_PATH_REGISTRY
pub static TASK_PATH_REGISTRY: LazyLock<Mutex<HashMap<usize, Vec<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// 2. Function to look up a task's path by ID
pub fn lookup_task_path(task_id: usize) -> Option<Vec<String>> {
    let registry = TASK_PATH_REGISTRY.lock();
    registry.get(&task_id).cloned()
}

// 3. Function to dump the entire registry
#[allow(dead_code)]
pub fn dump_task_path_registry() {
    debug_log!("==== TASK PATH REGISTRY DUMP ====");
    let task_paths = TASK_PATH_REGISTRY.lock().clone();
    debug_log!("Total registered tasks: {}", task_paths.len());

    let mut v = task_paths
        .iter()
        .map(|(&task_id, path)| (task_id, path.join("::")))
        .collect::<Vec<(usize, String)>>();

    v.sort();

    for (task_id, path) in &v {
        debug_log!("Task {}: {}", task_id, path);
    }
    drop(task_paths);
    debug_log!("=================================");
    flush_debug_log();
}

// 4. Utility function to look up and print a specific task's path
#[allow(dead_code)]
pub fn print_task_path(task_id: usize) {
    match lookup_task_path(task_id) {
        Some(path) => debug_log!("Task {} path: {}", task_id, path.join("::")),
        None => debug_log!("No path registered for task {}", task_id),
    }
    flush_debug_log();
}

// 5. Function to remove an entry from the TASK_PATH_REGISTRY
#[allow(dead_code)]
pub fn remove_task_path(task_id: usize) {
    let mut registry = TASK_PATH_REGISTRY.lock();
    registry.remove(&task_id);
}

// Helper function to find the best matching task_id
pub fn find_matching_task_id(path: &[String]) -> usize {
    let path_registry = TASK_PATH_REGISTRY.lock();
    // For each active profile, compute a similarity score
    let mut best_match = 0;
    let mut best_score = 0;
    let path_len = path.len();

    // debug_log!("get_active_tasks()={:#?}", get_active_tasks());
    #[allow(unused_assignments)]
    let mut score = 0;
    for task_id in get_active_tasks().iter().rev() {
        if let Some(reg_path) = path_registry.get(task_id) {
            score = compute_similarity(path, reg_path);
            if score > best_score || score == path_len {
                best_score = score;
                best_match = *task_id;
            }
            if score == path_len {
                break;
            }
        }
    }

    // Return the best match if found, otherwise fall back to last active task
    if best_match > 0 {
        return best_match;
    }

    // Fallback: Return the most recently activated profile
    debug_log!("...returning fallback: most recently activated profile - for path: {path:?}");
    get_last_active_task().unwrap_or(0)
}

// Compute similarity between a task path and backtrace frames
fn compute_similarity(task_path: &[String], reg_path: &[String]) -> usize {
    if task_path.is_empty() || reg_path.is_empty() {
        debug_log!("task_path.is_empty() || reg_path.is_empty()");
        return 0;
    }

    let score = task_path
        .iter()
        .zip(reg_path.iter())
        .filter(|(path_func, frame)| frame == path_func)
        .count();

    if score == 0 {
        debug_log!("score = {score} for path of length {}", task_path.len());
        debug_log!("{}\n{}", task_path.join("->"), reg_path.join("->"));
    }

    score
}

// ========== MEMORY PROFILING LIFECYCLE ==========

/// Initialize memory profiling.
/// This is called by the main `init_profiling` function.
#[allow(clippy::missing_panics_doc)]
pub fn initialize_memory_profiling() {
    // Set up allocator state with Tracking as the default
    USING_SYSTEM_ALLOCATOR.store(false, Ordering::SeqCst);

    // Use system allocator just for logging
    with_sys_alloc(|| {
        debug_log!("Memory profiling initialized");
        flush_debug_log();
    });
    #[cfg(debug_assertions)]
    assert_eq!(current_allocator(), Allocator::Tracking);
}

/// Finalize memory profiling and write out data.
/// This is called by the main `finalize_profiling` function.
pub fn finalize_memory_profiling() {
    write_memory_profile_data();
    // write_memory_dealloc_data();
    flush_debug_log();
}

/// Write memory profile data to a file
#[allow(clippy::too_many_lines)]
fn write_memory_profile_data() {
    use std::{collections::HashMap, fs::File, path::Path};

    with_sys_alloc(|| {
        // Retrieve registries to get task allocations and names
        let memory_path = get_memory_path().unwrap_or("memory.folded");

        // Check if the file exists first
        let file_exists = Path::new(memory_path).exists();

        // If the file already exists, write the summary information to the existing file
        // Otherwise, create a new file with the appropriate headers
        let file_result = if file_exists {
            debug_log!("Opening existing file in append mode");
            File::options().append(true).open(memory_path)
        } else {
            debug_log!("Creating new file");
            match File::create(memory_path) {
                Ok(file) => {
                    // // Write headers similar to time profile file
                    // if let Err(e) = writeln!(file, "# Memory Profile") {
                    //     debug_log!("Error writing header: {e}");
                    //     return;
                    // }

                    // if let Err(e) = writeln!(
                    //     file,
                    //     "# Script: {}",
                    //     std::env::current_exe().unwrap_or_default().display()
                    // ) {
                    //     debug_log!("Error writing script path: {e}");
                    //     return;
                    // }

                    // if let Err(e) =
                    //     writeln!(file, "# Started: {}", START_TIME.load(Ordering::SeqCst))
                    // {
                    //     debug_log!("Error writing date: {e}");
                    //     return;
                    // }

                    // if let Err(e) = writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION")) {
                    //     debug_log!("Error writing version: {e}");
                    //     return;
                    // }

                    // if let Err(e) = writeln!(file) {
                    //     debug_log!("Error writing newline: {e}");
                    //     return;
                    // }

                    Ok(file)
                }
                Err(e) => {
                    debug_log!("Error creating file: {e}");
                    Err(e)
                }
            }
        };

        if let Ok(file) = file_result {
            let mut writer = io::BufWriter::new(file);

            // Get the task path registry mapping for easier lookup
            let task_paths_map: HashMap<usize, Vec<String>> = {
                let binding = TASK_PATH_REGISTRY.lock();

                // Dump all entries for debugging
                // for (id, path) in binding.iter() {
                //     debug_log!("Registry entry: task {id}: path: {:?}", path);
                // }

                // Get all entries from the registry
                binding
                    .iter()
                    .map(|(task_id, path)| (*task_id, path.clone()))
                    .collect()
            };

            let mut already_written = HashSet::new();

            // Now write all tasks from registry that might not have allocations
            // This helps with keeping the full call hierarchy in the output
            for (task_id, path) in &task_paths_map {
                let task_id = *task_id;

                // let path_str = path.join(";");
                let path_str = build_stack(path, None, ";");
                if already_written.contains(&path_str) {
                    continue;
                }

                debug_log!("Writing for task {task_id} from registry: '{path_str}' with 0 bytes");

                // Write line with zero bytes to maintain call hierarchy
                write_alloc(task_id, 0, &mut writer, &mut already_written, &path_str);
            }

            // Make sure to flush the writer
            if let Err(e) = writer.flush() {
                debug_log!("Error flushing writer: {e}");
            }
        }
    });
}

fn write_alloc(
    task_id: usize,
    allocation: usize,
    writer: &mut io::BufWriter<std::fs::File>,
    already_written: &mut HashSet<String>,
    path_str: &str,
) {
    match writeln!(writer, "{} {}", path_str, allocation) {
        Ok(()) => {
            already_written.insert(path_str.to_string());
        }
        Err(e) => debug_log!("Error writing line for task {task_id}: {e}"),
    }
}
