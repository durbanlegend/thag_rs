/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features=["full_profiling", "debug_logging"] }
*/

use std::time::Instant;

use thag_profiler::{enable_profiling, end, profile};

fn clean_function_name_orig(clean_name: &mut str) -> String {
    let mut clean_name: &mut str = if let Some(closure_pos) = clean_name.find("::{{closure}}") {
        &mut clean_name[..closure_pos]
    } else if let Some(hash_pos) = clean_name.rfind("::h") {
        if clean_name[hash_pos + 3..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        {
            &mut clean_name[..hash_pos]
        } else {
            clean_name
        }
    } else {
        clean_name
    };

    while clean_name.ends_with("::") {
        let len = clean_name.len();
        clean_name = &mut clean_name[..len - 2];
    }

    let mut clean_name = (*clean_name).to_string();
    while clean_name.contains("::::") {
        clean_name = clean_name.replace("::::", "::");
    }

    clean_name
}

fn clean_function_name_opt(name: &mut str) -> String {
    let trimmed = if let Some(pos) = name.find("::{{closure}}") {
        &name[..pos]
    } else if let Some(pos) = name.rfind("::h") {
        let hex = &name[pos + 3..];
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            &name[..pos]
        } else {
            name
        }
    } else {
        name
    };

    let trimmed = trimmed.trim_end_matches("::");

    let mut result = String::with_capacity(trimmed.len());
    let mut chars = trimmed.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' && chars.peek() == Some(&':') {
            while chars.peek() == Some(&':') {
                chars.next();
            }
            result.push_str("::");
        } else {
            result.push(c);
        }
    }

    result
}

#[enable_profiling]
fn main() {
    let samples = [
        "my_crate::module::::my_fn::{{closure}}",
        "another::module::some_fn::h4d2c6a7b9e8f1c3a",
        "just::some::::weird::path::::::",
        "regular::path::to::function",
        "path::with::::multiple::::colons::hdeadbeef",
        "ends::with::double_colon::",
    ];

    const ITERATIONS: usize = 10_000;

    // Original version
    let start = Instant::now();
    profile!(original);
    for _ in 0..ITERATIONS {
        for s in &samples {
            let mut s = s.to_string();
            let _ = clean_function_name_orig(&mut s);
        }
    }
    end!(original);
    let duration_orig = start.elapsed();

    // Optimized version
    profile!(optimized);
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        for s in &samples {
            let mut s = s.to_string();
            let _ = clean_function_name_opt(&mut s);
        }
    }
    end!(optimized);
    let duration_opt = start.elapsed();

    println!("Original duration: {:?}", duration_orig);
    println!("Optimized duration: {:?}", duration_opt);
}
