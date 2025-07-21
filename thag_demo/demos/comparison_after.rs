/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - AFTER version with efficient implementations
/// This demo demonstrates the "after" state for differential profiling
//# Purpose: Demonstrate efficient implementations for before/after comparison
//# Categories: profiling, demo, comparison, optimization, after
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn sort(mut arr: Vec<i32>) -> Vec<i32> {
    // Efficient quicksort implementation - O(n log n)
    if arr.len() <= 1 {
        return arr;
    }

    let pivot = arr.len() / 2;
    let pivot_value = arr[pivot];
    let len = arr.len();
    arr.swap(pivot, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if arr[j] < pivot_value {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, len - 1);

    let (left, right) = arr.split_at_mut(i);
    let (pivot_slice, right) = right.split_at_mut(1);

    let mut left_sorted = sort(left.to_vec());
    let mut right_sorted = sort(right.to_vec());

    left_sorted.extend_from_slice(pivot_slice);
    left_sorted.extend_from_slice(&right_sorted);
    left_sorted
}

#[profiled]
fn string_concat(words: &[&str]) -> String {
    // Efficient string concatenation with pre-allocated capacity
    let mut result = String::with_capacity(words.len() * 10); // Pre-allocate
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}

#[profiled]
fn lookup(data: &HashMap<String, i32>, key: &str) -> Option<i32> {
    // Efficient HashMap lookup - O(1) average case
    data.get(key).copied()
}

#[profiled]
fn demonstrate_sorting() {
    println!("🔄 Sorting Algorithm Test (Efficient)");
    println!("{}", "─".repeat(31));

    let test_data: Vec<i32> = (0..1000).rev().collect(); // Reverse sorted (worst case)

    println!("Testing quicksort (O(n log n))...");
    let _sorted = sort(test_data.clone());

    println!("Sorting test completed!");
    println!();
}

#[profiled]
fn demonstrate_string_operations() {
    println!("📝 String Concatenation Test (Efficient)");
    println!("{}", "─".repeat(35));

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

    println!("Testing efficient string concatenation...");
    let _result = string_concat(&test_words);

    println!("String concatenation test completed!");
    println!();
}

#[profiled]
fn demonstrate_lookup_operations() {
    println!("🔍 Data Lookup Test (Efficient)");
    println!("{}", "─".repeat(26));

    // Prepare test data as HashMap
    let mut hashmap_data = HashMap::new();
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = i * 2;
        hashmap_data.insert(key, value);
    }

    let search_key = "key_500";

    println!("Testing HashMap O(1) lookup...");
    for _ in 0..100 {
        let _result = lookup(&hashmap_data, search_key);
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
    println!("⚖️  Performance Comparison Demo - AFTER (Efficient)");
    println!("{}", "═".repeat(51));
    println!();

    println!("Running efficient algorithm implementations...");
    println!("This represents the 'after' state for differential analysis.");
    println!();

    run_all_tests();

    println!("✅ After demo completed!");
    println!("📊 Profile data generated for differential comparison.");
    println!("🔍 This will be compared against the 'before' version.");
    println!("🎯 Expect to see O(n log n) sorting, pre-allocated strings, and HashMap lookups!");
}
