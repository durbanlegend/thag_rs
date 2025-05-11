# Fixing the Profiler Memory Tracking Issues

## Core Problems Identified

1. The profiler's memory tracking relies on unsafe access to thread-related state 
2. The current implementation appears to panic during async thread transitions
3. The application hangs without producing output or logs

## Guiding Principles

- Follow the design constraint: no TLS in thag_profiler
- Preserve the global state architecture 
- Focus on practical fixes, not architectural rewrites

## Recommended Approach

### 1. Modify the Global Allocator Implementation

The main dispatcher should use a simpler method to determine which allocator to use:

```rust
unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // For profiling code, explicitly mark as using System allocator
        // For user code, use TaskAware allocator
        // Use a simple heuristic based on stack traces rather than thread ID
        
        let using_system = is_profiler_stack();
        
        match using_system {
            true => self.system.alloc(layout),
            false => self.task_aware.alloc(layout),
        }
    }
}
```

### 2. Replace Thread-based Identification

Instead of using `std::thread::current()`, replace it with backtrace analysis:

```rust
fn is_profiler_stack() -> bool {
    // Get a backtrace and check if it contains profiler code
    // This avoids thread identity confusion in async contexts
    let bt = backtrace::Backtrace::new_unresolved();
    let frames = bt.frames();
    
    // Check if any frame matches profiler-related functions
    frames.iter().any(|frame| {
        let symbols = frame.symbols();
        symbols.iter().any(|symbol| {
            if let Some(name) = symbol.name() {
                let name = name.to_string();
                name.contains("thag_profiler") || 
                name.contains("mem_tracking") ||
                name.contains("profiling")
            } else {
                false
            }
        })
    })
}
```

### 3. Implement Explicit Allocator Guards

Use explicit function wrappers for critical code:

```rust
pub fn with_system_allocator<F, R>(f: F) -> R
where 
    F: FnOnce() -> R
{
    // Set a global flag indicating system allocator should be used
    USING_SYSTEM_ALLOCATOR.store(true, Ordering::SeqCst);
    
    // Run the function
    let result = f();
    
    // Restore the flag
    USING_SYSTEM_ALLOCATOR.store(false, Ordering::SeqCst);
    
    result
}
```

### 4. Simplify Thread Handling in Debug Logging

Make debug logging more robust against thread termination:

```rust
pub fn debug_log(message: &str) {
    // Write to the log without relying on thread identity
    // Use simple atomic operations for thread safety
    if let Some(log) = get_debug_log() {
        // Log without attempting to access current thread info
        writeln!(log, "{}: {}", current_timestamp(), message);
    }
}
```

### 5. Use Timeouts for Critical Resources

Add timeout protection for mutex acquisition:

```rust
pub fn with_timeout<F, R>(timeout: Duration, f: F) -> Option<R>
where
    F: FnOnce() -> R
{
    // Create a channel for the result
    let (sender, receiver) = std::sync::mpsc::channel();
    
    // Spawn a thread to run the function
    std::thread::spawn(move || {
        let result = f();
        let _ = sender.send(result);
    });
    
    // Wait for the result with timeout
    receiver.recv_timeout(timeout).ok()
}
```

## Implementation Steps

1. Implement a simple stack-based allocator detection
2. Remove all uses of `std::thread::current()` for identification
3. Add atomic flags for explicit allocator control
4. Update logging to be thread-termination safe
5. Add timeouts for critical operations

This approach maintains the global state design while addressing the fundamental issues that cause hanging and panics in async contexts.