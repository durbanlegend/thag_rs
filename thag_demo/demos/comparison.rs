/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["time_profiling"] }

[profile.release]
debug = true
strip = false
*/

/// Comparison demo - shows how to use thag_profiler for before/after performance comparison
/// This demo demonstrates differential profiling features of thag_profiler
//# Purpose: Demonstrate before/after performance comparison with thag_profiler
//# Categories: profiling, demo, comparison, optimization
use std::collections::HashMap;
use thag_profiler::{enable_profiling, profiled};

#[profiled]
fn bubble_sort_inefficient(mut arr: Vec<i32>) -> Vec<i32> {
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
fn quicksort_efficient(mut arr: Vec<i32>) -> Vec<i32> {
    if arr.len() <= 1 {
        return arr;
    }

    let pivot = arr.len() / 2;
    let pivot_value = arr[pivot];
    arr.swap(pivot, arr.len() - 1);

    let mut i = 0;
    for j in 0..arr.len() - 1 {
        if arr[j] < pivot_value {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, arr.len() - 1);

    let (left, right) = arr.split_at_mut(i);
    let (pivot_slice, right) = right.split_at_mut(1);

    let mut left_sorted = quicksort_efficient(left.to_vec());
    let mut right_sorted = quicksort_efficient(right.to_vec());

    left_sorted.extend_from_slice(pivot_slice);
    left_sorted.extend_from_slice(&right_sorted);
    left_sorted
}

#[profiled]
fn string_concatenation_naive(words: &[&str]) -> String {
    let mut result = String::new();
    for word in words {
        result = result + word + " ";
    }
    result
}

#[profiled]
fn string_concatenation_efficient(words: &[&str]) -> String {
    let mut result = String::with_capacity(words.len() * 10); // Pre-allocate
    for word in words {
        result.push_str(word);
        result.push(' ');
    }
    result
}

#[profiled]
fn map_lookup_vector(data: &[(String, i32)], key: &str) -> Option<i32> {
    for (k, v) in data {
        if k == key {
            return Some(*v);
        }
    }
    None
}

#[profiled]
fn map_lookup_hashmap(data: &HashMap<String, i32>, key: &str) -> Option<i32> {
    data.get(key).copied()
}

#[profiled]
fn demonstrate_sorting_comparison() {
    println!("🔄 Sorting Algorithm Comparison");
    println!("-------------------------------");

    let test_data: Vec<i32> = (0..1000).rev().collect(); // Reverse sorted (worst case)

    println!("Testing bubble sort (O(n²))...");
    let _sorted1 = bubble_sort_inefficient(test_data.clone());

    println!("Testing quicksort (O(n log n))...");
    let _sorted2 = quicksort_efficient(test_data.clone());

    println!("Sorting comparison completed!");
    println!();
}

#[profiled]
fn demonstrate_string_comparison() {
    println!("📝 String Concatenation Comparison");
    println!("----------------------------------");

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
    let _result1 = string_concatenation_naive(&test_words);

    println!("Testing efficient string concatenation...");
    let _result2 = string_concatenation_efficient(&test_words);

    println!("String concatenation comparison completed!");
    println!();
}

#[profiled]
fn demonstrate_lookup_comparison() {
    println!("🔍 Data Lookup Comparison");
    println!("-------------------------");

    // Prepare test data
    let mut vector_data = Vec::new();
    let mut hashmap_data = HashMap::new();

    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = i * 2;
        vector_data.push((key.clone(), value));
        hashmap_data.insert(key, value);
    }

    let search_key = "key_500";

    println!("Testing vector linear search...");
    for _ in 0..100 {
        let _result = map_lookup_vector(&vector_data, search_key);
    }

    println!("Testing HashMap O(1) lookup...");
    for _ in 0..100 {
        let _result = map_lookup_hashmap(&hashmap_data, search_key);
    }

    println!("Lookup comparison completed!");
    println!();
}

#[profiled]
fn run_all_comparisons() {
    demonstrate_sorting_comparison();
    demonstrate_string_comparison();
    demonstrate_lookup_comparison();
}

#[enable_profiling(time)]
fn main() {
    println!("⚖️  Performance Comparison Demo");
    println!("===============================");
    println!();

    println!("Running performance comparisons with profiling...");
    println!("This demo compares inefficient vs efficient algorithms.");
    println!();

    run_all_comparisons();

    println!("✅ Demo completed!");
    println!("📊 Check the generated flamegraph files to see performance differences.");
    println!("🔍 Use 'thag_profile' command to analyze the comparative performance.");
    println!("💡 The wide bars show slow operations, narrow bars show fast operations.");
    println!("🎯 Look for the dramatic difference between O(n²) and O(n log n) algorithms!");
}
