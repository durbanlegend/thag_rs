/// Demo of an advanced generic macro to generate lazy static variables.
/// See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.
//# Purpose: Demonstrate a handy alternative to the `lazy_static` crate.
//# Categories: macros, technique
// use std::sync::OnceLock;

/// A generic macro for lazily initializing a static variable using `OnceLock`.
///
/// # Parameters
/// - `$static_var`: The static variable name.
/// - `$init_fn`: The initialization function, which is only called once.
/// - $name: todo()
///
/// # Example
/// ```rust
/// let my_lazy_var = lazy_static_var!(HashMap<usize, &'static str>, { /* initialization */ });
/// ```
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

fn main() {
    println!("1. Simple Case with Explicit Type");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<i32>, {
            eprintln!("Initializing simple case...");
            vec![1, 2, 3]
        });
        println!("Value: {:?}", my_lazy_var);
    }

    println!("\n2. Debug Logging with Explicit Type");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<i32>, {
            println!("Initialising");
            vec![4, 5, 6]
        });
        println!("Value: {:?}", my_lazy_var);
    }

    println!("\n3. Explicit Type with Logging");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<usize>, { vec![7, 8, 9] });
        println!("Value: {:?}", my_lazy_var);
    }

    println!("\n4. Dereferenced Value");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<i32>, { vec![10, 11, 12] });
        println!("Value: {:?}", my_lazy_var);
    }

    println!("\n5. Debug Logging + Dereference");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<i32>, { vec![13, 14, 15] });
        println!("Value: {:?}", *my_lazy_var);
    }
}
