/// Demo of an advanced generic macro to generate lazy static variables.
/// See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.
//# Purpose: Demonstrate a handy alternative to the `lazy_static` crate.
//# Categories: macros, technique

// A generic macro for lazily initializing a static variable using `OnceLock`.
//
// This macro provides a way to initialize static variables lazily. The initialization
// function is guaranteed to be called only once, even in concurrent contexts.
//
// # Parameters
//
// - `$type`: The type of the static variable. Ensure that the type matches the return type of the initialization expression.
// - `$init_fn`: The initialization function or expression, which is executed only once.
// - `$name` (optional): A debug name for the variable, used in logging (if provided).
// - `deref` (optional): If specified, the macro dereferences the initialized value. Useful for types that implement `Deref`.
//
// # Behavior
//
// - **Single Initialization**: Once initialized, subsequent accesses return the same value without re-running the initialization logic.
// - **Dereferencing**: For types that support `Deref`, the macro can return a dereferenced version of the value (e.g., tuples).
// - **Debug Logging**: If a `$name` is provided, the macro logs the initialization using `eprintln!`.
//
// # Examples
//
// ## Simple Initialization
// ```rust
// let my_lazy_var = lazy_static_var!(Vec<i32>, vec![1, 2, 3]);
// println!("Lazy value: {:?}", my_lazy_var);
// ```
//
// ## Initialization with Debug Logging
// ```rust
// let my_lazy_var = lazy_static_var!(Vec<i32>, vec![4, 5, 6], "MyLazyVec");
// println!("Lazy value: {:?}", my_lazy_var);
// ```
//
// ## Initialization with Dereferencing
// ```rust
// let my_lazy_var = lazy_static_var!((usize, usize, usize), (10, 11, 12), deref);
// println!("Dereferenced lazy value: {:?}", my_lazy_var);
// ```
//
// ## Advanced Example with Error Handling
// ```rust
// let my_lazy_var = lazy_static_var!(
//     Result<usize, &'static str>,
//     Err("Initialization failed"),
//     "ErrorProneLazyVar"
// );
// match my_lazy_var {
//     Ok(value) => println!("Initialized value: {}", value),
//     Err(e) => eprintln!("Failed to initialize: {}", e),
// }
// ```
#[macro_export]
macro_rules! lazy_static_var {
    // With error handling (returning a Result)
    ($type:ty, $init_fn:expr, $name:expr, result) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| {
            eprintln!("Initializing lazy static variable: {}", $name);
            match $init_fn {
                Ok(value) => value,
                Err(e) => panic!("Lazy static initialization failed: {}", e),
            }
        })
    }};
    // With type, debug name, and dereference
    ($type:ty, $init_fn:expr , $name:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| {
            eprintln!("Initializing lazy static variable: {}", $name);
            $init_fn
        })
    }};
    // With type and dereference (no debug name)
    ($type:ty, $init_fn:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| {
            eprintln!("Initializing lazy static variable");
            $init_fn
        })
    }};
    // With type and debug name
    ($type:ty, $init_fn:expr, $name:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| {
            eprintln!("Initializing lazy static variable: {}", $name);
            $init_fn
        })
    }};
    // With type only (no debug name or dereference)
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| {
            eprintln!("Initializing lazy static variable");
            $init_fn
        })
    }};
}

fn main() {
    let my_lazy_var = lazy_static_var!(
        Result<usize, &'static str>,
        Err("Initialization failed"),
        "ErrorProneLazyVar",
        result
    );

    // The macro will panic if initialization fails.
    println!("Lazy value: {:?}", my_lazy_var);
}
