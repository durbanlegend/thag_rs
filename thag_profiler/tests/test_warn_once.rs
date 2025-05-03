#![cfg(test)]
use thag_profiler::warn_once;

/// Tests for the warn_once macro and warn_once_with_id function
/// 
/// These tests verify that the warning suppression mechanisms work correctly
/// by ensuring warning functions are only called once.

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    #[test]
    fn test_warn_once_macro() {
        // Counter for function calls
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        // Condition that's always true
        let condition = true;
        
        // Call warn_once multiple times
        for _ in 0..5 {
            warn_once!(condition, || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            });
        }
        
        // The warning function should only be called once
        assert_eq!(*counter.lock().unwrap(), 1, "Warning function should only be called once");
    }
    
    #[test]
    fn test_warn_once_with_return() {
        // Counter for function calls
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        // Variable to track early returns
        let mut returns = 0;
        
        // Call warn_once with early return multiple times
        for _ in 0..5 {
            warn_once!(true, || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            }, {
                returns += 1;
                continue;
            });
            
            // This should never execute due to the continue in the return expression
            panic!("This should not be reached");
        }
        
        // The warning function should only be called once
        assert_eq!(*counter.lock().unwrap(), 1, "Warning function should only be called once");
        // But we should have returned 5 times
        assert_eq!(returns, 5, "Early return should happen on every iteration");
    }
    
    #[test]
    fn test_warn_once_false_condition() {
        // Counter for function calls
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        
        // Call warn_once with false condition
        for _ in 0..5 {
            warn_once!(false, || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
            });
        }
        
        // The warning function should never be called
        assert_eq!(*counter.lock().unwrap(), 0, "Warning function should not be called when condition is false");
    }
    
    #[test]
    fn test_warn_once_with_id() {
        // Counter for function calls
        let counter1 = Arc::new(Mutex::new(0));
        let counter2 = Arc::new(Mutex::new(0));
        
        // Call warn_once_with_id with two different IDs
        for _ in 0..5 {
            let counter1_clone = counter1.clone();
            let counter2_clone = counter2.clone();
            
            // Safe to use in test context with controlled IDs
            unsafe {
                // First ID
                thag_profiler::mem_tracking::warn_once_with_id(1, true, || {
                    let mut count = counter1_clone.lock().unwrap();
                    *count += 1;
                });
                
                // Second ID - should be independent
                thag_profiler::mem_tracking::warn_once_with_id(2, true, || {
                    let mut count = counter2_clone.lock().unwrap();
                    *count += 1;
                });
            }
        }
        
        // Each warning function should only be called once
        assert_eq!(*counter1.lock().unwrap(), 1, "First warning function should only be called once");
        assert_eq!(*counter2.lock().unwrap(), 1, "Second warning function should only be called once");
    }
}