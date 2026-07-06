/*[toml]
[dependencies]
thag_profiler = { version = "1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - BEFORE version with inefficient implementations
/// This demo demonstrates the "before" state for differential profiling
//# Purpose: Demonstrate inefficient implementations for before/after comparison
//# Categories: profiling, demo, comparison, optimization, before
use thag_profiler::{enable_profiling, profiled};

// #[profiled] Not profiled directly because comparison_after.rs counterpart can't be because recursive.
fn sort(mut arr: Vec<i32>) -> Vec<i32> {
    // Inefficient bubble sort implementation - O(n²)
    let n = arr.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
    arr
}

#[profiled]
fn string_concat(words: &[&str]) -> String {
    // Inefficient string concatenation - causes multiple reallocations
    let mut result = String::new();
    for word in words {
        result = result + word + " ";
    }
    result
}

#[profiled]
fn lookup(data: &[(String, i32)], key: &str) -> Option<i32> {
    // Inefficient linear search through vector - O(n)
    for (k, v) in data {
        if k == key {
            return Some(*v);
        }
    }
    None
}

#[profiled]
fn demonstrate_sorting() {
    println!("🔄 Sorting Algorithm Test (Inefficient)");
    println!("{}", "─".repeat(33));

    let test_data: Vec<i32> = (0..1000).rev().collect(); // Reverse sorted (worst case)

    println!("Testing bubble sort (O(n²))...");
    let _sorted = sort(test_data.clone());

    println!("Sorting test completed!");
    println!();
}

#[profiled]
fn demonstrate_string_operations() {
    println!("📝 String Concatenation Test (Inefficient)");
    println!("{}", "─".repeat(37));

    let words = vec![
        "hello",
        "world",
        "this",
        "is",
        "a",
        "test",
        "of",
        "string",
        "performance",
    ];
    let test_words: Vec<&str> = words.iter().cycle().take(1000).copied().collect();

    println!("Testing naive string concatenation...");
    let _result = string_concat(&test_words);

    println!("String concatenation test completed!");
    println!();
}

#[profiled]
fn demonstrate_lookup_operations() {
    println!("🔍 Data Lookup Test (Inefficient)");
    println!("{}", "─".repeat(28));

    // Prepare test data as vector of tuples
    let mut vector_data = Vec::new();
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = i * 2;
        vector_data.push((key, value));
    }

    let search_key = "key_500";

    println!("Testing vector linear search...");
    for _ in 0..100 {
        let _result = lookup(&vector_data, search_key);
    }

    println!("Lookup test completed!");
    println!();
}

#[profiled]
fn run_all_tests() {
    demonstrate_sorting();
    demonstrate_string_operations();
    demonstrate_lookup_operations();
}

#[enable_profiling(time)]
fn main() {
    println!("⚖️  Performance Comparison Demo - BEFORE (Inefficient)");
    println!("{}", "═".repeat(54));
    println!();

    println!("Running inefficient algorithm implementations...");
    println!("This represents the 'before' state for differential analysis.");
    println!();

    run_all_tests();

    println!("✅ Before demo completed!");
    println!("📊 Profile data generated for differential comparison.");
    println!("🔍 This will be compared against the 'after' version.");
    println!("🎯 Expect to see O(n²) sorting, string reallocations, and linear searches!");
}
