/// Demo of a generic macro to generate lazy static variables.
/// Sometimes you need to call a function repeatedly and it makes sense for it to lazily initialise a
/// variable that it will use each time. I got you fam!
///
/// See also `demo/macro_lazy_static_var_advanced.rs` for a more advanced form of the macro.
//# Purpose: Demonstrate a handy alternative to the `lazy_static` crate.
use std::collections::HashMap;

// A generic macro for lazily initializing a static variable using `OnceLock`.
//
// # Parameters
// - `$type`: The type of the static variable.
// - `$init_fn`: The initialization function, which is only called once.
// - `deref` (optional): Dereferences the initialized value for direct access.
//
// # Example
// ```rust
// let my_lazy_var = lazy_static_var!(HashMap<usize, &'static str>, { /* initialization */ });
// ```
#[macro_export]
macro_rules! lazy_static_var {
    ($type:ty, $init_fn:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}

fn get_error_description(code: usize) -> &'static str {
    let lazy_static_error_map = lazy_static_var!(HashMap<usize, &'static str>, {
        eprintln!("Generating the error map - you should only see this function being called once");
        let mut errors = HashMap::new();
        errors.insert(404, "Not Found");
        errors.insert(403, "Forbidden");
        errors.insert(500, "Internal Server Error");
        errors.insert(502, "Bad Gateway");
        errors.insert(503, "Service Unavailable");
        errors
    });
    lazy_static_error_map
        .get(&code)
        .copied()
        .unwrap_or("Unknown Error")
}

fn main() {
    // Simulate multiple function calls to demonstrate lazy initialization
    let codes = vec![404, 500, 401, 503];
    for code in codes {
        println!("Error {}: {}", code, get_error_description(code));
    }
}
