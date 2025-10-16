#!/usr/bin/env thag
//# Purpose: Test shared target directory and executable cache logic
//# Categories: testing, demo

/// A simple demo script to test the new shared target directory and executable cache.
///
/// This script uses `serde_json` as a dependency to verify that:
/// 1. Dependencies are compiled once and shared across all scripts
/// 2. The executable is cached in the executable cache directory
/// 3. Subsequent runs are fast due to warm cache
///
/// Run with: `thag demo/test_shared_cache.rs`
/// Then run again to see the speed improvement from caching.
/// Clean cache with: `thag --clean` or `thag --clean bins`
use serde_json::{json, Value};

fn main() {
    println!("Testing shared target and executable cache logic");
    println!("=========================================================\n");

    // Create some JSON data to demonstrate the dependency is working
    let data = json!({
        "script": "test_shared_cache.rs",
        "purpose": "Test shared target directory",
        "features": [
            "Shared dependency cache",
            "Executable caching",
            "Faster builds"
        ],
        "directories": {
            "shared_target": "$TMPDIR/thag_rs_shared_target",
            "executable_cache": "$TMPDIR/thag_rs_bins"
        }
    });

    println!("Configuration:");
    println!("{}\n", serde_json::to_string_pretty(&data).unwrap());

    // Some simple computation
    let features: Vec<String> = data["features"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .map(String::from)
        .collect();

    println!("Features enabled:");
    for (i, feature) in features.iter().enumerate() {
        println!("  {}. {}", i + 1, feature);
    }

    println!("\n✓ Script executed successfully!");
    println!("\nTip: Run this script again to see improved performance from caching.");
    println!("     Use 'thag --clean' to clear all caches.");
}
