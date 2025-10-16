//! # Procedural Macros Documentation
//!
//! ## category_enum
//!
//! Generates a `Category` enum with predefined variants and utility implementations.
//!
//! The `category_enum` macro defines an enum `Category` with a hardcoded set of variants.
//! This ensures consistency across all callers and centralizes control over the available categories.
//!
//! Additionally, it generates:
//! - A `FromStr` implementation to parse strings into the `Category` enum.
//! - A utility method `Category::all_categories()` to return a list of all available category names.
//!
//! # Usage
//!
//! Simply invoke the macro in your project:
//!
//! ```rust
//! use thag_proc_macros::category_enum;
//!
//! category_enum!();
//! ```
//!
//! This generates:
//!
//! ```rust
//! pub enum Category {
//!     AST,
//!     CLI,
//!     REPL,
//!     Async,
//!     Basic,
//!     BigNumbers,
//!     Crates,
//!     ErrorHandling,
//!     Exploration,
//!     Filesystem,
//!     Learning,
//!     Macros,
//!     Math,
//!     ProcMacros,
//!     Prototype,
//!     Recreational,
//!     Reference,
//!     Technique,
//!     Testing,
//!     Tools,
//!     TypeIdentification,
//! }
//!
//! impl std::str::FromStr for Category {
//!     type Err = String;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         match s {
//!             "AST" => Ok(Category::AST),
//!             "CLI" => Ok(Category::CLI),
//!             "REPL" => Ok(Category::REPL),
//!             "Async" => Ok(Category::Async),
//!             // ... other variants ...
//!             _ => Err(format!("Invalid category: {s}")),
//!         }
//!     }
//! }
//!
//! impl Category {
//!     pub fn all_categories() -> Vec<&'static str> {
//!         vec![
//!             "AST", "CLI", "REPL", "Async", "Basic", "BigNumbers", "Crates",
//!             "ErrorHandling", "Exploration", "Filesystem", "Learning", "Macros", "Math",
//!             "ProcMacros", "Prototype", "Recreational", "Reference", "Technique",
//!             "Testing", "Tools", "TypeIdentification",
//!         ]
//!     }
//! }
//! ```
//!
//! # Benefits
//!
//! - Consistency: The hardcoded list ensures uniformity across all callers.
//! - Convenience: Auto-generated utility methods simplify working with the categories.
//! - Safety: Enums prevent invalid values at compile time.
//!
//! # Use Cases
//!
//! This macro is ideal for scenarios requiring centralized control over predefined categories,
//! such as filtering demo scripts or generating reports.
