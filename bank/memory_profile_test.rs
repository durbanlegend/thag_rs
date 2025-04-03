/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/
use std::{thread, time::Duration};
use thag_profiler::{
    create_memory_task, enable_profiling, finalize_profiling, init_profiling, profiled,
};

#[thag_profiler::enable_profiling]
fn main() {
    println!("Starting memory profile test");
    allocate_some_memory();
    println!("Test completed");

    #[thag_profiler::profiled]
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

    // println!("Allocating memory...");

    // Allocate in a loop
    let mut data = Vec::new();
    for i in 0..10 {
        let chunk = vec![i as u8; 1024 * (i + 1)];
        data.push(chunk);

        // Give the allocator tracking a moment to catch up
        thread::sleep(Duration::from_millis(50));

        // // Check memory usage
        // if let Some(usage) = task.memory_usage() {
        //     println!("Memory usage after allocation {}: {} bytes", i, usage);
        // } else {
        //     println!("Memory usage not available for task ID: {}", task.id());

        //     // Check again with task ID
        //     println!("  Task ID: {}", task.id());
        // }
    }

    println!("Allocation complete!");

    // // Make sure we don't drop the data before checking memory usage
    // let total_size = data.iter().map(|v| v.len()).sum::<usize>();
    // println!("Total data size: {} bytes", total_size);

    // // Check one more time
    // if let Some(usage) = task.memory_usage() {
    //     println!("Final memory usage: {} bytes", usage);
    // } else {
    //     println!("Final memory usage not available");
    // }

    // // Finalize profiling
    // println!("Finalizing profiling");
    // finalize_profiling();
    // println!("Profiling finalized");
}
