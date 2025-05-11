# Non-TLS Approach for Allocator Switching in thag_profiler

## Current Problem

The profiler currently uses thread-local storage (TLS) to track which allocator should be used (System vs TaskAware) for different operations. This approach causes problems in:

1. Async contexts where tasks move between threads
2. During thread shutdown/unwinding
3. With alternative thread scheduling models

## Proposed Solution

Replace TLS-based allocator selection with an explicit context-based approach that works across thread boundaries.

## Implementation Approach

### 1. Context ID-based Allocation

Instead of using thread identity to determine the allocator context, use an explicit context ID:

```rust
pub struct AllocatorContext {
    id: usize,
    allocator_type: Allocator,
}
```

### 2. Global Context Registry

Maintain a global registry of active allocator contexts:

```rust
pub struct ContextRegistry {
    contexts: HashMap<usize, Allocator>,
    next_id: AtomicUsize,
}

// Global instance protected by a mutex
static CONTEXT_REGISTRY: Lazy<Mutex<ContextRegistry>> = Lazy::new(|| 
    Mutex::new(ContextRegistry::new())
);
```

### 3. Context Stack for Each Operation

Keep a global "current operation" context:

```rust
static CURRENT_OPERATION: AtomicUsize = AtomicUsize::new(0);

// Push a context for the current operation
pub fn push_allocator_context(allocator: Allocator) -> ContextGuard {
    let registry = CONTEXT_REGISTRY.lock();
    let op_id = registry.next_id.fetch_add(1, Ordering::SeqCst);
    registry.contexts.insert(op_id, allocator);
    CURRENT_OPERATION.store(op_id, Ordering::SeqCst);
    ContextGuard { op_id }
}

// RAII guard that restores previous context when dropped
pub struct ContextGuard {
    op_id: usize,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        let mut registry = CONTEXT_REGISTRY.lock();
        registry.contexts.remove(&self.op_id);
        // Restore previous context if any
        let prev_id = registry.contexts.keys().max().copied().unwrap_or(0);
        CURRENT_OPERATION.store(prev_id, Ordering::SeqCst);
    }
}
```

### 4. Safe Context Switching

Replace TLS-based context with explicit context switching:

```rust
// Function to run with system allocator
pub fn with_system_allocator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = push_allocator_context(Allocator::System);
    f()
}

// Function to run with task-aware allocator
pub fn with_task_allocator<F, R>(task_id: usize, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = push_allocator_context(Allocator::TaskAware);
    f()
}
```

### 5. Dispatcher Implementation

Update the global allocator dispatcher to check the operation context:

```rust
unsafe impl GlobalAlloc for Dispatcher {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get current context from global registry
        let current = {
            let op_id = CURRENT_OPERATION.load(Ordering::SeqCst);
            if op_id == 0 {
                // No active context, default to TaskAware
                Allocator::TaskAware
            } else {
                let registry = CONTEXT_REGISTRY.lock();
                registry.contexts.get(&op_id).copied().unwrap_or(Allocator::TaskAware)
            }
        };

        match current {
            Allocator::TaskAware => self.task_aware.alloc(layout),
            Allocator::System => self.system.alloc(layout),
        }
    }
    
    // Similar for dealloc...
}
```

### 6. Memory Task Context Integration

Integrate with existing memory task tracking:

```rust
impl TaskMemoryContext {
    pub fn enter(&self) -> Result<TaskGuard, ProfileError> {
        // When entering a task context, switch to TaskAware allocator
        let _guard = push_allocator_context(Allocator::TaskAware);
        
        // Rest of existing implementation...
        // ...
        
        Ok(TaskGuard { task_id: self.task_id })
    }
}
```

## Benefits of This Approach

1. **Thread Independence**: Works across thread boundaries
2. **Explicit Control**: Clear control over allocator context
3. **Async Friendly**: Compatible with async task migration
4. **No TLS Dependencies**: Works with any threading model
5. **Clear Lifetime**: RAII guards ensure proper context cleanup

## Implementation Plan

1. Refactor basic context tracking without changing task memory attribution
2. Update the dispatcher to use the new context system
3. Integrate with the task system
4. Update profile creation to use explicit context switching
5. Update tests to verify thread-independence