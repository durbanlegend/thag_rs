# Manual Fix for Thread Panic in mem_tracking.rs

The issue causing the panic at line 66 in `mem_tracking.rs` is related to unsafe thread-local storage (TLS) access in async contexts.

## Step 1: Locate the problematic function

Find the `initialize_thread_allocator_context` function in `thag_rs/thag_profiler/src/mem_tracking.rs`.

## Step 2: Replace with this safer implementation

```rust
pub fn initialize_thread_allocator_context() {
    // Get inherited context safely
    let inherited = match std::panic::catch_unwind(|| {
        CURRENT_ALLOCATOR_CONTEXT.with(|cell| cell.get())
    }) {
        Ok(allocator) => allocator,
        Err(_) => Allocator::System, // Default to system allocator if TLS access fails
    };
    
    // Access thread-local state safely
    let _ = std::panic::catch_unwind(|| {
        ALLOCATOR_STATE.with(|state| {
            if let Ok(mut state) = state.try_borrow_mut() {
                if state.1 == 0 {
                    state.0 = inherited;
                    
                    // Get thread info safely
                    let thread_id = std::thread::current().id();
                    let thread_name = std::thread::current().name().unwrap_or("unnamed");
                    
                    debug_log!(
                        "Initializing thread ({:?}/{}) allocator context to {:?} from parent thread",
                        thread_id,
                        thread_name,
                        inherited
                    );
                }
            }
        });
    });
}
```

This modification:
1. Uses `catch_unwind` to safely handle TLS access during thread termination
2. Uses `try_borrow_mut` instead of `borrow_mut` to avoid panics if already borrowed
3. Properly accesses thread information without causing double-borrows

## Step 3: Also update current_allocator function

Find the `current_allocator` function and replace it with:

```rust
pub fn current_allocator() -> Allocator {
    // Try to initialize safely
    let _ = std::panic::catch_unwind(|| {
        initialize_thread_allocator_context();
    });
    
    // Get allocator safely, defaulting to System if anything fails
    match std::panic::catch_unwind(|| {
        ALLOCATOR_STATE.with(|state| {
            if let Ok(borrow) = state.try_borrow() {
                borrow.0
            } else {
                Allocator::System
            }
        })
    }) {
        Ok(allocator) => allocator,
        Err(_) => Allocator::System,
    }
}
```

These changes make the thread-local state handling much more robust in async contexts.