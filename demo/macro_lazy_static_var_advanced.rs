/// Demo of an advanced generic macro to generate lazy static variables.
/// See also `demo/macro_lazy_static_var_errs.rs` for a more meaningful usage example.
//# Purpose: Demonstrate a handy alternative to the `lazy_static` crate.

/// A generic macro for lazily initializing a static variable using `OnceLock`.
///
/// # Parameters
/// - `$type`: The type of the static variable.
/// - `$init_fn`: The initialization function, which is only called once.
/// - `deref` (optional): Dereferences the initialized value for direct access.
///
/// # Example
/// ```rust
/// let my_lazy_var = lazy_static_var!(HashMap<usize, &'static str>, { /* initialization */ });
/// ```
#[macro_export]
macro_rules! lazy_static_var {
    // With type, debug name, and dereference
    ($type:ty, $init_fn:expr, $name:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        eprintln!("Initializing lazy static variable: {}", $name);
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    // With type and dereference (no debug name)
    ($type:ty, $init_fn:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    // With type and debug name
    ($type:ty, $init_fn:expr, $name:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        eprintln!("Initializing lazy static variable: {}", $name);
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    // With type only (no debug name or dereference)
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}

fn case_2() {
    let my_lazy_var = lazy_static_var!(Vec<i32>, { vec![4, 5, 6] }, "MyLazyVec");
    println!("Value: {:?}", my_lazy_var);
}

fn case_3() {
    let my_lazy_var = lazy_static_var!(Vec<usize>, { vec![7, 8, 9] }, "ExplicitTypeLazyVec");
    println!("Value: {:?}", my_lazy_var);
}

fn main() {
    println!("1. Simple Case with Explicit Type");
    for _ in 0..3 {
        // case_1();
        let my_lazy_var = lazy_static_var!(Vec<i32>, {
            eprintln!("Initializing simple case...");
            vec![1, 2, 3]
        });
        println!("Value: {:?}", my_lazy_var);
    }

    println!("\n2. Debug Logging with Explicit Type");
    for _ in 0..3 {
        case_2();
    }

    println!("\n3. Explicit Type with Logging");
    for _ in 0..3 {
        case_3();
    }

    println!("\n4. Dereferenced Value");
    for _ in 0..3 {
        let my_lazy_var = lazy_static_var!(Vec<i32>, { vec![10, 11, 12] }, deref);
        println!("Value: {:?}", *my_lazy_var);
    }

    println!("\n5. Debug Logging + Dereference");
    for _ in 0..3 {
        let my_lazy_var =
            lazy_static_var!(Vec<i32>, { vec![13, 14, 15] }, "MyDereferencedVec", deref);
        println!("Value: {:?}", *my_lazy_var);
    }
}
