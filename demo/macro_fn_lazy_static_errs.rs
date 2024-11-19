/// Demo of a generic macro to generate lazy static variables.
use std::collections::HashMap;

#[macro_export]
macro_rules! lazy_static_fn {
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

fn create_error_map() -> HashMap<usize, &'static str> {
    eprintln!("Generating the error map - you should only see this function being called once");
    let mut errors = HashMap::new();
    errors.insert(404, "Not Found");
    errors.insert(403, "Forbidden");
    errors.insert(500, "Internal Server Error");
    errors.insert(502, "Bad Gateway");
    errors.insert(503, "Service Unavailable");
    errors
}

fn get_error_description(code: usize) -> &'static str {
    let error_map = lazy_static_fn!(HashMap<usize, &'static str>, create_error_map());
    error_map.get(&code).copied().unwrap_or("Unknown Error")
}

fn main() {
    // Simulate multiple function calls to demonstrate lazy initialization
    let codes = vec![404, 500, 401, 503];
    for code in codes {
        println!("Error {}: {}", code, get_error_description(code));
    }
}
