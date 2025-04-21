#![allow(clippy::uninlined_format_args)]
#![deny(unsafe_op_in_unsafe_fn)]
//! Task-aware memory allocator for profiling.
//!
//! This module provides a memory allocator that tracks allocations by logical tasks
//! rather than threads, making it suitable for async code profiling. It also contains
//! the custom memory allocator implementation that enables memory profiling.

use crate::{
    debug_log, extract_path, find_profile, flush_debug_log, get_global_profile_type,
    is_detailed_memory, lazy_static_var,
    profiling::{
        clean_function_name, extract_alloc_callstack, extract_detailed_alloc_callstack,
        get_memory_detail_dealloc_path, get_memory_detail_path, get_memory_path, get_reg_desc_name,
        is_profiling_state_enabled, MemoryDetailDeallocFile, MemoryDetailFile, MemoryProfileFile,
        START_TIME,
    },
    regex, Profile, ProfileRef, ProfileType,
};
use backtrace::{Backtrace, BacktraceFrame};
use parking_lot::Mutex;
use regex::Regex;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::{BTreeSet, HashMap, HashSet},
    env,
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        LazyLock,
    },
    thread::{self, ThreadId},
    time::Instant,
};

pub static ALLOC_START_PATTERN: LazyLock<&'static Regex> =
    LazyLock::new(|| regex!("thag_profiler::task_allocator.+Dispatcher"));

// ========== MEMORY ALLOCATOR DEFINITION ==========

/// Enum defining all available allocators used by the profiler
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Allocator {
    /// Default allocator that performs memory tracking
    TaskAware,
    /// System allocator used internally to prevent recursion
    System,
}

// We'll use a simple atomic instead of TLS to avoid destructors
static CURRENT_ALLOCATOR_ID: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

// Function to get current allocator type
pub fn current_allocator() -> Allocator {
    let allocator_id = CURRENT_ALLOCATOR_ID.load(std::sync::atomic::Ordering::Relaxed);
    if allocator_id == 0 {
        Allocator::TaskAware
    } else {
        Allocator::System
    }
}

// Function to run code with a specific allocator
pub fn with_allocator<T, F: FnOnce() -> T>(req_alloc: Allocator, f: F) -> T {
    // Convert allocator enum to ID
    let allocator_id = match req_alloc {
        Allocator::TaskAware => 0,
        Allocator::System => 1,
    };

    // Save the current allocator
    let prev_id = CURRENT_ALLOCATOR_ID.load(std::sync::atomic::Ordering::Relaxed);

    // Set the new allocator
    CURRENT_ALLOCATOR_ID.store(allocator_id, std::sync::atomic::Ordering::Relaxed);

    // Run the function
    let result = f();

    // Restore the previous allocator
    CURRENT_ALLOCATOR_ID.store(prev_id, std::sync::atomic::Ordering::Relaxed);

    result
}

/// Task-aware allocator that tracks memory allocations
pub struct TaskAwareAllocator;

// Static instance for global access
static TASK_AWARE_ALLOCATOR: TaskAwareAllocator = TaskAwareAllocator;

// Helper to get the allocator instance
pub fn get_allocator() -> &'static TaskAwareAllocator {
    &TASK_AWARE_ALLOCATOR
}

#[allow(clippy::unused_self)]
impl TaskAwareAllocator {
    /// Creates a new task context for tracking memory
    pub fn create_task_context(&'static self) -> TaskMemoryContext {
        let task_id = TASK_STATE.next_task_id.fetch_add(1, Ordering::SeqCst);

        // Initialize in profile registry
        activate_task(task_id);

        TaskMemoryContext { task_id }
    }
}

unsafe impl GlobalAlloc for TaskAwareAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        with_allocator(Allocator::System, || {
            let ptr = unsafe { System.alloc(layout) };

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
        with_allocator(Allocator::System, || {
            if !ptr.is_null() && is_profiling_state_enabled() {
                // Potentially skip small allocations
                let size = layout.size();
                if size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_dealloc(address, size);
                }
            }

            // Forward to system allocator for deallocation
            unsafe { System.dealloc(ptr, layout) };
        });
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if !ptr.is_null() && is_profiling_state_enabled() {
            with_allocator(Allocator::System, || {
                // Potentially skip small allocations
                let dealloc_size = layout.size();
                if dealloc_size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_dealloc(address, dealloc_size);
                }

                // Potentially skip small allocations
                if new_size > *SIZE_TRACKING_THRESHOLD {
                    let address = ptr as usize;
                    record_alloc(address, new_size);
                }
            });
        }
        ptr
    }
}

#[allow(clippy::too_many_lines)]
fn record_alloc(address: usize, size: usize) {
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

    if size == 0 {
        debug_log!("Zero-sized allocation found");
        return;
    }

    let profile_type = get_global_profile_type();
    if profile_type != ProfileType::Memory && profile_type != ProfileType::Both {
        debug_log!(
            "Skipping allocation recording because profile_type={:?}",
            profile_type
        );
        return;
    }

    // Flag if we're already tracking in case it causes an infinite recursion
    let in_tracking = unsafe { IN_TRACKING };

    #[cfg(debug_assertions)]
    assert!(!in_tracking);

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
    let mut task_id = 0;
    // Now we can safely use backtrace without recursion!
    // debug_log!("Calling extract_callstack");
    let mut current_backtrace = Backtrace::new();

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

    let detailed_memory = lazy_static_var!(bool, deref, is_detailed_memory());
    let file_names = {
        crate::mem_attribution::PROFILE_REGISTRY
            .lock()
            .get_file_names()
    };
    debug_log!("file_names={file_names:#?}");

    // let Some((filename, lineno, frame, fn_name, profile_ref)) = Backtrace::frames(&current_backtrace)
    let func_and_ancestors: Vec<(String, u32, String, String, ProfileRef)> =
        Backtrace::frames(&current_backtrace)
            .iter()
            .flat_map(BacktraceFrame::symbols)
            .map(|symbol| (symbol.filename(), symbol.lineno(), symbol.name()))
            // .inspect(|(maybe_filename, maybe_lineno, frame)| {
            //     debug_log!("maybe_filename: {maybe_filename:?}, maybe_lineno: {maybe_lineno:?}, frame: {frame:?}");
            // })
            .filter(|(maybe_filename, maybe_lineno, frame)| {
                maybe_filename.is_some()
                    && maybe_lineno.is_some()
                    && frame.is_some()
                    && !frame.as_ref().unwrap().to_string().starts_with('<')
            })
            .map(|(maybe_filename, maybe_lineno, maybe_frame)| {
                (
                    maybe_filename
                        .unwrap()
                        .to_owned()
                        .as_path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    maybe_lineno.unwrap(),
                    maybe_frame.unwrap().to_string(),
                )
            })
            // .inspect(|(filename, lineno, frame)| {
            //     debug_log!("filename: {filename:?}, lineno: {lineno:?}, frame: {frame:?}, file_names={file_names:?}");
            // })
            .filter(|(filename, _, _)| (file_names.contains(filename)))
            // .inspect(|(_, _, _)| {
            //     // debug_log!("filename: {filename:?}, lineno: {lineno:?}, frame: {frame:?}, file_names={file_names:?}");
            //     debug_log!("***File names match");
            // })
            .map(|(filename, lineno, mut frame)| {
                (
                    filename,
                    lineno,
                    frame.clone(),
                    clean_function_name(frame.as_mut_str()),
                )
            })
            .map(|(filename, lineno, frame, fn_name)| {
                (
                    filename.clone(),
                    lineno,
                    frame,
                    fn_name.clone(),
                    find_profile(&filename, &fn_name, lineno),
                )
            })
            // .inspect(|(_, _, _, _, maybe_profile_ref)| {
            //     debug_log!("maybe_profile_ref={maybe_profile_ref:?}");
            // })
            .filter(|(_, _, _, _, maybe_profile_ref)| maybe_profile_ref.is_some())
            .map(|(filename, lineno, frame, fn_name, maybe_profile_ref)| {
                (filename, lineno, frame, fn_name, maybe_profile_ref.unwrap())
            })
            // .map(|(filename, lineno, frame| (filename, lineno, frame.to_string()))
            // .cloned()
            .collect();
    // .last() else {return};

    if func_and_ancestors.is_empty() {
        debug_log!("No eligible profile found");
        return;
    }

    let in_profile_code = func_and_ancestors.iter().any(|(_, _, frame, _, _)| {
        frame.contains("Backtrace::new") || frame.contains("Profile::new")
    });

    if in_profile_code {
        debug_log!("Ignoring allocation request of size {size} for profiler code");
        return;
    }

    let (filename, lineno, frame, fn_name, _profile_ref) = &func_and_ancestors[0];

    debug_log!(
        "Found filename (file_name)={filename}, lineno={lineno}, fn_name: {fn_name:?}, frame: {frame:?}"
    );

    // Try to record the allocation in the new profile registry
    if !filename.is_empty()
        && *lineno > 0
        && crate::mem_attribution::record_allocation(
            filename,
            fn_name,
            *lineno,
            size,
            address,
            &mut current_backtrace,
        )
    {
        debug_log!(
            "Recorded allocation of {size} bytes in {filename}::{fn_name}:{lineno} to a profile"
        );

        // Still record detailed allocations to -memory_detail.folded if requested
        if detailed_memory {
            write_detailed_alloc(size, &ALLOC_START_PATTERN, &mut current_backtrace, true);
        }

        // Allocation was recorded in a profile, no need to continue with global tracking
        return;
    }

    // Fall back to traditional method
    current_backtrace.resolve();

    let cleaned_stack = extract_alloc_callstack(&ALLOC_START_PATTERN, &mut current_backtrace);
    debug_log!("Cleaned_stack for size={size}: {cleaned_stack:?}");

    if cleaned_stack.is_empty() {
        debug_log!(
            "...empty cleaned_stack for backtrace: size={size}:\n{:#?}",
            trim_backtrace(&ALLOC_START_PATTERN, &current_backtrace)
        );
        debug_log!("Getting last active task (hmmm :/)");
        task_id = get_last_active_task().unwrap_or(0);
    } else {
        // Make sure the use of a separate allocator is working.
        assert!(!cleaned_stack
            .iter()
            .any(|frame| frame.contains("find_matching_profile")));

        // debug_log!("Calling extract_path");
        let path = extract_path(&cleaned_stack, None);
        // debug_log!("path={path:#?}");
        if path.is_empty() {
            let trimmed_backtrace = trim_backtrace(&ALLOC_START_PATTERN, &current_backtrace);
            if trimmed_backtrace
                .iter()
                .any(|frame| frame.contains("Backtrace::new"))
            {
                debug_log!("Ignoring setup allocation of size {size} containing Backtrace::new:");
                return;
            }
            debug_log!(
                "...path is empty for thread {:?}, size: {size:?}",
                thread::current().id(),
            );
        } else {
            task_id = find_matching_task_id(&path);
            debug_log!("...find_matching_task_id found task_id={task_id} for size={size}");
        }
    }
    debug_log!(
        "task_id={task_id}, size={size}, time to assign = {}ms",
        start_ident.elapsed().as_millis()
    );

    // // Record allocation if task found
    // if task_id == 0 {
    //     // TODO if necessary, record in suspense file and allocate later
    //     return;
    // }

    record_alloc_for_task_id(address, size, task_id);

    if file_names.is_empty() {
        return;
    }

    if detailed_memory {
        write_detailed_alloc(size, &ALLOC_START_PATTERN, &mut current_backtrace, true);
    }
}

pub fn record_alloc_for_task_id(address: usize, size: usize, task_id: usize) {
    let start_record_alloc = Instant::now();

    debug_log!("Recording allocation for task_id={task_id}, address={address:#x}, size={size}");
    let mut registry = ALLOC_REGISTRY.lock();
    registry
        .task_allocations
        .entry(task_id)
        .or_default()
        .push((address, size));

    registry.address_to_task.insert(address, task_id);

    if cfg!(debug_assertions) {
        let check_map = &registry.task_allocations;
        let reg_task_id = *registry.address_to_task.get(&address).unwrap();
        let maybe_vec = check_map.get(&task_id);
        let (addr, sz) = *maybe_vec
            .and_then(|v: &Vec<(usize, usize)>| {
                let last = v.iter().filter(|&(addr, _)| *addr == address).last();
                last
            })
            .unwrap();
        drop(registry);
        assert_eq!(sz, size);
        assert_eq!(addr, address);
        assert_eq!(reg_task_id, task_id);
    }

    debug_log!(
        "Time to record allocation: {}ms",
        start_record_alloc.elapsed().as_millis()
    );
}

pub fn write_detailed_alloc(
    size: usize,
    start_pattern: &Regex,
    current_backtrace: &mut Backtrace,
    write_to_detail_file: bool,
) {
    let detailed_stack = extract_detailed_alloc_callstack(start_pattern, current_backtrace);

    write_detailed_stack_alloc(size, write_to_detail_file, &detailed_stack);
}

#[allow(clippy::ptr_arg)]
pub fn write_detailed_stack_alloc(
    size: usize,
    write_to_detail_file: bool,
    detailed_stack: &Vec<String>,
) {
    let entry = if detailed_stack.is_empty() {
        // format!("[out_of_bounds] +{}", size)
        format!("[out_of_bounds] +{size}")
    } else {
        let descr_stack = &detailed_stack
            .iter()
            .map(|raw_str| get_reg_desc_name(raw_str).unwrap_or_else(|| raw_str.to_string()))
            .collect::<Vec<String>>()
            .join(";");

        format!("{} +{size}", descr_stack)
    };

    let (memory_path, file) = if write_to_detail_file {
        (get_memory_detail_path().unwrap(), MemoryDetailFile::get())
    } else {
        (get_memory_path().unwrap(), MemoryProfileFile::get())
    };
    let _ = Profile::write_profile_event(memory_path, file, &entry);
}

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

    let profile_type = get_global_profile_type();
    if profile_type != ProfileType::Memory && profile_type != ProfileType::Both {
        debug_log!(
            "Skipping deallocation recording because profile_type={:?}",
            profile_type
        );
        return;
    }

    // Flag if we're already tracking in case it causes an infinite recursion
    let in_tracking = unsafe { IN_TRACKING };

    #[cfg(debug_assertions)]
    assert!(!in_tracking);

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
    let start_pattern: &Regex = regex!("thag_profiler::task_allocator.+Dispatcher");

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
            // debug_log!(
            //     "...empty detailed_stack for backtrace: size={size}:\n{:#?}",
            //     // trim_backtrace(start_pattern, &current_backtrace)
            //     current_backtrace
            // );
            // Assuming address is not volatile
            // let address = ptr as usize;

            let reg_task_id = {
                *ALLOC_REGISTRY
                    .lock()
                    .address_to_task
                    .get(&address)
                    .unwrap_or(&0)
            };
            // Caution: more condensed
            let reg_path: Vec<String> = {
                TASK_PATH_REGISTRY
                    .lock()
                    .get(&reg_task_id)
                    .unwrap_or(&Vec::new())
                    .clone()
            };

            format!("{} +{size}", reg_path.join(";"))
        } else {
            format!("{} +{size}", detailed_stack.join(";"))
        };

        let memory_detail_dealloc_path = get_memory_detail_dealloc_path().unwrap();
        let _ = Profile::write_profile_event(
            memory_detail_dealloc_path,
            MemoryDetailDeallocFile::get(),
            &entry,
        );
    }
}

/// Dispatcher allocator that routes allocation requests to the appropriate allocator
pub struct Dispatcher {
    task_aware: TaskAwareAllocator,
    system: System,
}

impl Dispatcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            task_aware: TaskAwareAllocator,
            system: System,
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
        // Get current allocator type from thread-local storage
        match current_allocator() {
            Allocator::TaskAware => unsafe { self.task_aware.alloc(layout) },
            Allocator::System => unsafe { self.system.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // // For deallocation, we don't need to check the allocator type, as both
        // // allocators use System for the actual deallocation.
        // unsafe { self.system.dealloc(ptr, layout) };
        match current_allocator() {
            Allocator::TaskAware => unsafe { self.task_aware.dealloc(ptr, layout) },
            Allocator::System => unsafe { self.system.dealloc(ptr, layout) },
        }
    }
}

// Create a direct static instance
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

/// Registry for tracking memory allocations and deallocations
#[derive(Debug)]
pub struct AllocationRegistry {
    /// Task ID -> Allocations mapping: [(address, size)]
    pub task_allocations: HashMap<usize, Vec<(usize, usize)>>,
    /// Task ID -> Deallocations mapping: [(address, size)]
    pub task_deallocations: HashMap<usize, Vec<(usize, usize)>>,

    /// Address -> Task ID mapping for deallocations
    pub address_to_task: HashMap<usize, usize>,
}

impl AllocationRegistry {
    fn new() -> Self {
        Self {
            task_allocations: HashMap::new(),
            task_deallocations: HashMap::new(),
            address_to_task: HashMap::new(),
        }
    }

    /// Get the memory usage for a specific task
    fn get_task_memory_usage(&self, task_id: usize) -> Option<usize> {
        self.task_allocations
            .get(&task_id)
            .map(|allocations| allocations.iter().map(|(_, size)| *size).sum())
    }

    // /// Get the memory deallocations for a specific task
    // fn get_task_memory_deallocs(&self, task_id: usize) -> Option<usize> {
    //     self.task_deallocations
    //         .get(&task_id)
    //         .map(|deallocs| deallocs.iter().map(|(_, size)| *size).sum())
    // }
}

// Global allocation registry
pub static ALLOC_REGISTRY: LazyLock<Mutex<AllocationRegistry>> =
    LazyLock::new(|| Mutex::new(AllocationRegistry::new()));

/// Registry for tracking active profiles and task stacks
#[derive(Debug)]
struct ProfileRegistry {
    /// Set of active task IDs
    active_profiles: BTreeSet<usize>,

    /// Thread ID -> Stack of active task IDs (most recent on top)
    thread_task_stacks: HashMap<ThreadId, Vec<usize>>,
}

impl ProfileRegistry {
    fn new() -> Self {
        Self {
            active_profiles: BTreeSet::new(),
            thread_task_stacks: HashMap::new(),
        }
    }

    /// Add a task to active profiles
    fn activate_task(&mut self, task_id: usize) {
        self.active_profiles.insert(task_id);
    }

    /// Remove a task from active profiles
    fn deactivate_task(&mut self, task_id: usize) {
        self.active_profiles.remove(&task_id);
    }

    /// Get a copy of the active task IDs
    fn get_active_tasks(&self) -> Vec<usize> {
        self.active_profiles.iter().copied().collect()
    }

    /// Get the last (most recently added) active task
    fn get_last_active_task(&self) -> Option<usize> {
        self.active_profiles.iter().next_back().copied()
    }

    /// Add a task to a thread's stack
    fn push_task_to_stack(&mut self, thread_id: ThreadId, task_id: usize) {
        let stack = self.thread_task_stacks.entry(thread_id).or_default();
        stack.push(task_id);
    }

    /// Remove a task from a thread's stack
    fn pop_task_from_stack(&mut self, thread_id: ThreadId, task_id: usize) {
        if let Some(stack) = self.thread_task_stacks.get_mut(&thread_id) {
            if let Some(pos) = stack.iter().position(|id| *id == task_id) {
                stack.remove(pos);

                // Remove empty stack
                if stack.is_empty() {
                    self.thread_task_stacks.remove(&thread_id);
                }
            }
        }
    }
}

// Global profile registry
static PROFILE_REGISTRY: LazyLock<Mutex<ProfileRegistry>> =
    LazyLock::new(|| Mutex::new(ProfileRegistry::new()));

// ========== PUBLIC REGISTRY API ==========

/// Add a task to active profiles
pub fn activate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().activate_task(task_id);
    });
}

/// Remove a task from active profiles
#[allow(dead_code)]
pub fn deactivate_task(task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().deactivate_task(task_id);
    });
}

/// Get the memory usage for a specific task
pub fn get_task_memory_usage(task_id: usize) -> Option<usize> {
    ALLOC_REGISTRY.lock().get_task_memory_usage(task_id)
}

/// Add a task to a thread's stack
pub fn push_task_to_stack(thread_id: ThreadId, task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY
            .lock()
            .push_task_to_stack(thread_id, task_id);
    });
}

/// Remove a task from a thread's stack
#[allow(dead_code)]
pub fn pop_task_from_stack(thread_id: ThreadId, task_id: usize) {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY
            .lock()
            .pop_task_from_stack(thread_id, task_id);
    });
}

/// Get active tasks
pub fn get_active_tasks() -> Vec<usize> {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().get_active_tasks()
    })
}

/// Get the last active task
#[must_use]
pub fn get_last_active_task() -> Option<usize> {
    with_allocator(Allocator::System, || {
        PROFILE_REGISTRY.lock().get_last_active_task()
    })
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

    /// Gets current memory usage for this task
    #[must_use]
    pub fn memory_usage(&self) -> Option<usize> {
        get_task_memory_usage(self.task_id)
    }

    /// Enter this task context for memory tracking
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered (TODO: do we need a Result wrapper?)
    pub fn enter(&self) -> crate::ProfileResult<TaskGuard> {
        // Push to thread stack
        push_task_to_stack(thread::current().id(), self.task_id);
        Ok(TaskGuard::new(self.task_id))
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
        // Use simpler approach with direct method calls
        // This avoids complex TLS interactions during thread shutdown

        // Run these operations with System allocator
        with_allocator(Allocator::System, || {
            // Remove from active profiles
            debug_log!("Deactivating task {}", self.task_id);
            PROFILE_REGISTRY.lock().deactivate_task(self.task_id);

            // Remove from thread stack
            PROFILE_REGISTRY
                .lock()
                .pop_task_from_stack(thread::current().id(), self.task_id);

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
pub fn initialize_memory_profiling() {
    // This is called at application startup to set up memory profiling
    with_allocator(Allocator::System, || {
        debug_log!("Memory profiling initialized");
        flush_debug_log();
    });
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

    with_allocator(Allocator::System, || {
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
                Ok(mut file) => {
                    // Write headers similar to time profile file
                    if let Err(e) = writeln!(file, "# Memory Profile") {
                        debug_log!("Error writing header: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(
                        file,
                        "# Script: {}",
                        std::env::current_exe().unwrap_or_default().display()
                    ) {
                        debug_log!("Error writing script path: {e}");
                        return;
                    }

                    if let Err(e) =
                        writeln!(file, "# Started: {}", START_TIME.load(Ordering::SeqCst))
                    {
                        debug_log!("Error writing date: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION")) {
                        debug_log!("Error writing version: {e}");
                        return;
                    }

                    if let Err(e) = writeln!(file) {
                        debug_log!("Error writing newline: {e}");
                        return;
                    }

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

            // Get all task allocations
            let task_allocs = { ALLOC_REGISTRY.lock().task_allocations.clone() };

            // Get the task path registry mapping for easier lookup
            let task_paths_map: HashMap<usize, Vec<String>> = {
                let binding = TASK_PATH_REGISTRY.lock();

                // Dump all entries for debugging
                for (id, path) in binding.iter() {
                    debug_log!("Registry entry: task {id}: path: {:?}", path);
                }

                // Get all entries from the registry
                binding
                    .iter()
                    .map(|(task_id, pat)| (*task_id, pat.clone()))
                    .collect()
            };

            let mut already_written = HashSet::new();

            // Write out any allocations for task 0, i.e. unassigned allocations
            if let Some(allocation) = { ALLOC_REGISTRY.lock().get_task_memory_usage(0) } {
                debug_log!("Writing for task 0 (unassigned) with {allocation} bytes");
                write_alloc(0, allocation, &mut writer, &mut already_written, "");
            }

            // Now write all tasks from registry that might not have allocations
            // This helps with keeping the full call hierarchy in the output
            for (task_id, path) in &task_paths_map {
                let task_id = *task_id;

                // Skip tasks we've already written
                if task_allocs.contains_key(&task_id) {
                    continue;
                }

                let path_str = path.join(";");
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
    match writeln!(writer, "{} +{}", path_str, allocation) {
        Ok(()) => {
            already_written.insert(path_str.to_string());
        }
        Err(e) => debug_log!("Error writing line for task {task_id}: {e}"),
    }
}
