/*[toml]
[dependencies]
regex = { version = "1.11.1", default-features = false }
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/
use backtrace::{Backtrace, BacktraceFrame};
use regex::Regex;
use std::{collections::HashSet, thread, time::Duration};
use thag_profiler::profiling::clean_function_name;
use thag_profiler::{
    debug_log, extract_path, find_matching_task_id, get_task_memory_usage, regex, strip_hex_suffix,
    with_allocator, Allocator,
};

#[thag_profiler::enable_profiling(runtime)]
fn main() {
    println!("Starting memory profile test");
    allocate_some_memory();
    println!("Test completed");

    // #[thag_profiler::profiled]
    fn main() {
        println!("Starting memory profile test");
        allocate_some_memory();
        println!("Test completed");
    }

    main();
}

#[thag_profiler::profiled]
fn allocate_some_memory() {
    // Initialize profiling before creating memory task
    // println!("Initializing profiling");
    // init_profiling();
    // println!("Profiling initialized");

    // Create a memory task for this function
    // let task = create_memory_task();
    // println!("Created memory task with ID: {}", task.id());

    // let _guard = task.enter().expect("Failed to enter task context");
    // println!("Entered task context");

    // // Check task ID
    // println!("Task ID: {}", task.id());

    let task_id = with_allocator(Allocator::System, || {
        let mut current_backtrace = Backtrace::new_unresolved();
        current_backtrace.resolve();
        debug_log!(
            "module_path!()={:?}, current_backtrace=\n{current_backtrace:#?}",
            module_path!()
        );

        let start_pattern: &Regex = regex!("backtrace::capture::Backtrace::new_unresolved");
        let end_point = "__rust_begin_short_backtrace";
        let mut already_seen = HashSet::new();

        // First, collect all relevant frames
        let callstack: Vec<String> = Backtrace::frames(&current_backtrace)
            .iter()
            .flat_map(BacktraceFrame::symbols)
            .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
            .skip_while(|frame| !start_pattern.is_match(frame))
            .skip(1)
            .take_while(|frame| !frame.contains(end_point))
            .inspect(|frame| {
                debug_log!("frame: {frame}");
            })
            .map(strip_hex_suffix)
            .map(|mut name| {
                // Remove hash suffixes and closure markers to collapse tracking of closures into their calling function
                clean_function_name(&mut name)
            })
            .filter(|name| {
                // Skip duplicate function calls (helps with the {{closure}} pattern)
                if already_seen.contains(name.as_str()) {
                    false
                } else {
                    already_seen.insert(name.clone());
                    true
                }
            })
            .collect();
        // debug_log!("Callstack: {callstack:#?}");
        // debug_log!("already_seen: {:#?}", already_seen);

        // Redefine end-point as inclusive
        let end_point = "memory_profile_test::main";

        let cleaned_stack: Vec<String> = callstack
            .iter()
            .rev()
            .skip_while(|frame| !frame.contains(end_point))
            .cloned()
            .collect();

        if cleaned_stack.is_empty() {
            debug_log!("Empty cleaned stack found");
            return None;
        }

        let fn_name = &cleaned_stack[0];

        // #[cfg(not(target_os = "windows"))]
        // let desc_fn_name = fn_name.to_string();

        // #[cfg(target_os = "windows")]
        // let desc_fn_name = fn_name; // Windows already highlights async functions

        let path = extract_path(&cleaned_stack, Some(fn_name));

        // let stack = path.join(";");
        debug_log!("In allocate_some_memory, path={path:#?}");

        Some(find_matching_task_id(&path))
    })
    // .unwrap_or(2);
    .unwrap();

    with_allocator(Allocator::System, || println!("Allocating memory..."));

    // Allocate in a loop
    let mut data = Vec::new();
    for i in 0..10 {
        let chunk = vec![i as u8; 1024 * (i + 1)];
        data.push(chunk);

        // Give the allocator tracking a moment to catch up
        thread::sleep(Duration::from_millis(50));

        // Check memory usage
        if let Some(usage) = get_task_memory_usage(task_id) {
            println!("Memory usage after allocation {}: {} bytes", i, usage);
        } else {
            println!("Memory usage not available for task_id: {task_id}");

            // Check again with task ID
            // println!("  Task ID: {}", task.id());
        }
    }

    with_allocator(Allocator::System, || {
        println!("Allocation complete");

        // Make sure we don't drop the data before checking memory usage
        let total_size = data.iter().map(|v| v.len()).sum::<usize>();
        println!("Total data size: {} bytes", total_size);

        // Check one more time
        if let Some(usage) = get_task_memory_usage(task_id) {
            println!("Final memory usage: {} bytes", usage);
        } else {
            println!("Final memory usage not available");
        }
    });
}
