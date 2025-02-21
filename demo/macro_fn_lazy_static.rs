/// Demo of a generic macro to generate lazy static variables without the `lazy_static` crate.
//# Purpose: Demonstrate a technique
//# Categories: learning, technique
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

fn number_names() -> Vec<(usize, String)> {
    eprintln!("Generating the Vec - you should only see this function being called once");
    let names = vec![
        "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];

    // Zip numbers with names
    (1..=10).zip(names.into_iter().map(String::from)).collect()
}

fn main() {
    // Accessing the lazy static data

    let num_text_vec: &'static Vec<(usize, String)> =
        lazy_static_fn!(Vec<(usize, String)>, number_names());

    println!("First invocation");
    for (number, name) in num_text_vec.iter() {
        println!("{}: {}", number, name);
    }

    println!("Second invocation");
    for (number, name) in num_text_vec.iter() {
        println!("{}: {}", number, name);
    }
}
