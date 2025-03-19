use thag_profiler::task_allocator::create_memory_task;

#[test]
fn test_basic_allocation() {
    // Create a memory task
    let task = create_memory_task();
    
    // Do some allocations
    let data1: Vec<u8> = vec\![0; 1024];
    let data2: Vec<u8> = vec\![0; 2048];
    
    // Verify we can query memory usage without a crash
    println\!("Memory usage: {:?}", task.memory_usage());
    
    // Keep data in scope
    println\!("Data sizes: {}, {}", data1.len(), data2.len());
}
