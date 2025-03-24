#![allow(dead_code)]
// String interning for efficient string storage and comparison
use lasso::{Rodeo, Spur};
use parking_lot::Mutex;
use std::sync::LazyLock;

// String interner for efficient string storage and comparison
#[cfg(feature = "full_profiling")]
static STRING_INTERNER: std::sync::LazyLock<Mutex<Rodeo>> =
    LazyLock::new(|| Mutex::new(Rodeo::new()));

// Type alias for interned paths
#[cfg(feature = "full_profiling")]
type InternedPath = Vec<Spur>;

// Function to intern a string
#[cfg(feature = "full_profiling")]
pub fn intern(s: &str) -> Spur {
    let mut interner = STRING_INTERNER.lock();
    interner.get_or_intern(s)
}

// Function to resolve an interned string ID back to a string
#[cfg(feature = "full_profiling")]
pub fn resolve(id: Spur) -> String {
    // Get a copy of the string to avoid lifetime issues
    let interner = STRING_INTERNER.lock();
    interner.resolve(&id).to_string()
}

// Function to convert a Vec<String> to an InternedPath
#[cfg(feature = "full_profiling")]
pub fn intern_path(path: &[String]) -> InternedPath {
    path.iter().map(|s| intern(s)).collect()
}

// Function to convert a Vec<Spur> to a Vec<String>
#[cfg(feature = "full_profiling")]
pub fn resolve_path(path: &[Spur]) -> Vec<String> {
    path.iter().map(|s| resolve(*s)).collect()
}

// // Update the path registry with interned strings
// #[allow(dead_code)]
// #[cfg(feature = "full_profiling")]
// pub fn register_task_path(task_id: usize, path: &[String]) {
//     let interned_path = intern_path(path);
//     let mut registry = TASK_PATH_REGISTRY.lock();
//     registry.insert(task_id, interned_path);
// }
