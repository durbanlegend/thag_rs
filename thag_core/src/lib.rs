pub mod code_utils;
pub mod config;
pub mod error;
pub mod logging;
pub mod macros;
pub mod profiling;
pub mod shared; // Make the module public

pub use error::{ThagError, ThagResult};
pub use profiling::Profile; // Re-export for convenience
