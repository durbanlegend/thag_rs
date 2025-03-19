#[cfg(feature = "full_profiling")]
fn main() {
    use thag_profiler::enable_profiling;
    use thag_profiler::profiled;
    use thag_profiler::task_allocator::create_memory_task;

    #[profiled]
    fn allocate_some_memory() {
        // Create a memory task for this function
        let task = create_memory_task();
        let _guard = task.enter().expect("Failed to enter task context");

        println!("Allocating memory...");

        // Allocate in a loop
        let mut data = Vec::new();
        for i in 0..10 {
            let chunk = vec![i as u8; 1024 * (i + 1)];
            data.push(chunk);

            // Check memory usage
            if let Some(usage) = task.memory_usage() {
                println!("Memory usage after allocation {}: {} bytes", i, usage);
            } else {
                println!("Memory usage not available");
            }
        }

        println!("Allocation complete!");
    }

    #[enable_profiling]
    fn main() {
        println!("Starting memory profile test");
        allocate_some_memory();
        println!("Test completed");
    }

    main();
}

#[cfg(not(feature = "full_profiling"))]
fn main() {
    println!("This example requires the full_profiling feature");
}
