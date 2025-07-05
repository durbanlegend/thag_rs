#![allow(dead_code)]
//! Example showing different approaches to internal documentation attributes.
//!
//! This example demonstrates the two ways to mark items as internal documentation:
//! 1. Using the `#[internal_doc]` macro (recommended)
//! 2. Using the manual `#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]` attribute
//!
//! Run with different features to see the difference:
//!
//! ```bash
//! # Public API docs (internal items hidden)
//! cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps
//!
//! # Internal docs (internal items visible)
//! cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal_docs --no-deps
//!
//! # Internal docs with private items (comprehensive)
//! cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal_docs --no-deps --document-private-items
//! ```

use thag_profiler::internal_doc;

/// This is a public API function that should always be visible.
///
/// Users of the library should see this function in the documentation.
pub fn public_api_function() {
    println!("This function is part of the public API");
}

/// This is an internal utility function using the `#[internal_doc]` macro.
///
/// This function is hidden from public API docs but visible in internal docs.
/// The `#[internal_doc]` macro is the recommended way to mark internal items.
#[internal_doc]
pub fn internal_utility_with_macro() {
    println!("This function is internal and uses the macro");
}

/// This is an internal utility function using the manual attribute.
///
/// This function is hidden from public API docs but visible in internal docs.
/// The manual attribute is more verbose but works in contexts where the macro isn't available.
#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
pub fn internal_utility_with_manual_attribute() {
    println!("This function is internal and uses the manual attribute");
}

/// This is a private function that's only visible with `--document-private-items`.
///
/// Private functions are never part of the public API, but they can be documented
/// for internal development purposes when using the `--document-private-items` flag.
fn private_implementation_detail() {
    println!("This function is private and only visible with --document-private-items");
}

/// A struct that demonstrates different visibility levels.
pub struct ExampleStruct {
    /// Public field - always visible
    pub public_field: String,

    /// Private field - only visible with --document-private-items
    private_field: i32,
}

impl ExampleStruct {
    /// Public constructor - always visible
    pub fn new(value: String) -> Self {
        Self {
            public_field: value,
            private_field: 42,
        }
    }

    /// Internal method using the macro - hidden from public API
    #[internal_doc]
    pub fn internal_method(&self) {
        println!("Internal method: {}", self.public_field);
    }

    /// Internal method using manual attribute - hidden from public API
    #[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
    pub fn another_internal_method(&self) {
        println!("Another internal method: {}", self.private_field);
    }

    /// Private method - only visible with --document-private-items
    fn private_method(&self) {
        println!("Private method called");
    }
}

/// Internal module using the macro
#[internal_doc]
pub mod internal_utilities {
    /// A function inside an internal module
    pub fn helper_function() {
        println!("Helper function in internal module");
    }
}

/// Internal module using manual attribute
#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
pub mod more_internal_utilities {
    /// Another function inside an internal module
    pub fn another_helper_function() {
        println!("Another helper function in internal module");
    }
}

/// Private module - only visible with --document-private-items
mod private_module {
    /// Private function in private module
    pub fn private_module_function() {
        println!("Function in private module");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        public_api_function();
        let example = ExampleStruct::new("test".to_string());
        assert_eq!(example.public_field, "test");
    }

    #[test]
    fn test_internal_functions() {
        internal_utility_with_macro();
        internal_utility_with_manual_attribute();

        let example = ExampleStruct::new("test".to_string());
        example.internal_method();
        example.another_internal_method();
    }
}

fn main() {
    println!("Documentation attributes example");

    // Demonstrate public API
    public_api_function();

    // Demonstrate internal utilities
    internal_utility_with_macro();
    internal_utility_with_manual_attribute();

    // Demonstrate struct usage
    let example = ExampleStruct::new("example".to_string());
    example.internal_method();
    example.another_internal_method();

    // Demonstrate internal modules
    internal_utilities::helper_function();
    more_internal_utilities::another_helper_function();

    println!("Example completed successfully!");
}
