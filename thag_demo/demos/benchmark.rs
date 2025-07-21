/*[toml]
[dependencies]
thag_profiler = { version = "0.1, thag-auto", features = ["full_profiling", "demo"] }

[profile.release]
debug = true
strip = false
*/

/// Comprehensive benchmark demo - shows how to use thag_profiler for detailed performance analysis
/// This demo demonstrates full profiling capabilities including time, memory, and detailed analysis
//# Purpose: Demonstrate comprehensive benchmark profiling with thag_profiler
//# Categories: profiling, demo, benchmark, performance
use rand::{rng, Rng};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Instant;
use thag_profiler::{enable_profiling, profiled, timing, visualization, AnalysisType, ProfileType};

#[derive(Debug, Serialize, Deserialize)]
struct DataPoint {
    id: u64,
    value: f64,
    category: String,
    timestamp: u64,
}

#[timing]
#[profiled(time, mem_summary)]
fn generate_test_data(count: usize) -> Vec<DataPoint> {
    let mut rng = rng();
    let categories = vec!["A", "B", "C", "D", "E"];

    (0..count)
        .map(|i| DataPoint {
            id: i as u64,
            value: rng.random_range(0.0..1000.0),
            category: categories[rng.random_range(0..categories.len())].to_string(),
            timestamp: rng.random_range(1_000_000..2_000_000),
        })
        .collect()
}

#[timing]
#[profiled(time, mem_detail)]
fn process_data_sequential(data: &[DataPoint]) -> HashMap<String, Vec<f64>> {
    let mut result = HashMap::new();

    for point in data {
        result
            .entry(point.category.clone())
            .or_insert_with(Vec::new)
            .push(point.value);
    }

    // Calculate statistics for each category
    for values in result.values_mut() {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    result
}

#[timing]
#[profiled(time, mem_summary)]
fn process_data_parallel(data: &[DataPoint]) -> HashMap<String, Vec<f64>> {
    use std::sync::Mutex;

    let result = Mutex::new(HashMap::new());

    data.par_iter().for_each(|point| {
        let mut map = result.lock().unwrap();
        map.entry(point.category.clone())
            .or_insert_with(Vec::new)
            .push(point.value);
    });

    let mut final_result = result.into_inner().unwrap();

    // Sort values in parallel
    final_result.par_iter_mut().for_each(|(_, values)| {
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    });

    final_result
}

#[timing]
#[profiled(time)]
fn compute_statistics(data: &HashMap<String, Vec<f64>>) -> HashMap<String, serde_json::Value> {
    data.iter()
        .map(|(category, values)| {
            let sum: f64 = values.iter().sum();
            let mean = sum / values.len() as f64;
            let median = values[values.len() / 2];
            let min = values.first().copied().unwrap_or(0.0);
            let max = values.last().copied().unwrap_or(0.0);

            let stats = json!({
                "count": values.len(),
                "mean": mean,
                "median": median,
                "min": min,
                "max": max,
                "sum": sum
            });

            (category.clone(), stats)
        })
        .collect()
}

#[timing]
#[profiled(time, mem_detail)]
fn serialize_results(stats: &HashMap<String, serde_json::Value>) -> String {
    serde_json::to_string_pretty(stats).unwrap_or_else(|_| "Serialization failed".to_string())
}

#[timing]
#[profiled(time)]
fn regex_processing(data: &[DataPoint]) -> Vec<String> {
    let pattern = Regex::new(r"[A-E]").unwrap();

    data.iter()
        .filter(|point| pattern.is_match(&point.category))
        .map(|point| format!("{}:{:.2}", point.category, point.value))
        .collect()
}

#[timing]
#[profiled(time)]
fn cryptographic_hashing(data: &[String]) -> Vec<String> {
    data.par_iter()
        .map(|item| {
            let mut hasher = Sha256::new();
            hasher.update(item.as_bytes());
            format!("{:x}", hasher.finalize())
        })
        .collect()
}

#[timing]
#[profiled(time, mem_detail)]
fn memory_intensive_operations(count: usize) -> Vec<Vec<u8>> {
    let mut buffers = Vec::new();

    for i in 0..count {
        // Create varying sizes of buffers
        let size = 1024 + (i % 1000) * 512;
        let mut buffer = vec![0u8; size];

        // Fill with some data
        for (j, byte) in buffer.iter_mut().enumerate() {
            *byte = (i + j) as u8;
        }

        buffers.push(buffer);
    }

    buffers
}

#[timing]
#[profiled(time)]
fn cpu_bound_computation(iterations: usize) -> f64 {
    let mut result = 0.0;

    for i in 0..iterations {
        let x = i as f64 / 1000.0;
        result += x.sin() * x.cos() + x.tan().abs();
    }

    result
}

#[timing]
#[profiled(time, mem_summary)]
fn comprehensive_benchmark() {
    println!("🚀 Starting comprehensive benchmark...");

    let start_time = Instant::now();

    // Data generation phase
    println!("📊 Phase 1: Generating test data...");
    let data = generate_test_data(5_000);
    println!("Generated {} data points", data.len());

    // Sequential processing phase
    println!("⏳ Phase 2: Sequential processing...");
    let sequential_result = process_data_sequential(&data);
    let sequential_stats = compute_statistics(&sequential_result);

    // Parallel processing phase
    println!("⚡ Phase 3: Parallel processing...");
    let parallel_result = process_data_parallel(&data);
    let parallel_stats = compute_statistics(&parallel_result);

    // Regex processing phase
    println!("🔍 Phase 4: Regex processing...");
    let regex_results = regex_processing(&data);
    println!("Processed {} regex matches", regex_results.len());

    // Cryptographic hashing phase
    println!("🔐 Phase 5: Cryptographic hashing...");
    let sample_data: Vec<String> = regex_results.into_iter().take(1000).collect();
    let hashes = cryptographic_hashing(&sample_data);
    println!("Generated {} hashes", hashes.len());

    // Memory intensive operations phase
    println!("🧠 Phase 6: Memory intensive operations...");
    let buffers = memory_intensive_operations(100);
    println!("Created {} buffers", buffers.len());

    // CPU bound computation phase
    println!("💻 Phase 7: CPU bound computation...");
    let computation_result = cpu_bound_computation(100_000);
    println!("Computation result: {:.6}", computation_result);

    // Serialization phase
    println!("📝 Phase 8: Serialization...");
    let serialized_sequential = serialize_results(&sequential_stats);
    let serialized_parallel = serialize_results(&parallel_stats);

    let total_time = start_time.elapsed();

    println!("✅ Benchmark completed in {:.2?}", total_time);
    println!("Sequential JSON length: {}", serialized_sequential.len());
    println!("Parallel JSON length: {}", serialized_parallel.len());

    // Display sample results
    println!("\n📋 Sample Results:");
    for (category, stats) in sequential_stats.iter().take(3) {
        println!("  {}: {}", category, stats);
    }
}

#[timing]
#[profiled(mem_detail)]
fn stress_test_allocations() {
    println!("🔥 Running allocation stress test...");

    let mut large_structures = Vec::new();

    for i in 0..5 {
        let mut map = HashMap::new();
        for j in 0..1000 {
            let key = format!("key_{}_{}", i, j);
            let value = vec![i as u8; j % 100 + 1];
            map.insert(key, value);
        }
        large_structures.push(map);
    }

    println!("Created {} large structures", large_structures.len());

    // Force some deallocations
    large_structures.truncate(2);

    println!("Stress test completed");
}

#[enable_profiling]
fn demo() {
    // Run the main benchmark
    comprehensive_benchmark();

    println!();
    println!("🧪 Running additional stress tests...");
    stress_test_allocations();
}

async fn run_analysis(profile_type: &ProfileType, analysis_type: &AnalysisType) {
    // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // because profile output is only available after that function completes.
    if let Err(e) =
        visualization::show_interactive_prompt("benchmark", profile_type, analysis_type).await
    {
        eprintln!(
            "⚠️ Could not show interactive {} visualization: {e}",
            profile_type.to_string().to_lowercase()
        );
    }
}

fn main() {
    println!("🏆 Comprehensive Benchmark Demo");
    println!("===============================");
    println!();

    println!("This demo runs a comprehensive benchmark to showcase all profiling features.");
    println!("It includes time profiling, memory tracking, and detailed performance analysis.");
    println!();

    demo();
    println!();
    println!("✅ All benchmarks completed!");
    println!("📊 Check the generated profile files for detailed analysis:");
    println!("   • Time flamegraphs (.svg files)");
    println!("   • Memory flamegraphs (.svg files)");
    println!("   • Profile data (.folded files)");
    println!("🔍 Use 'thag_profile' command to:");
    println!("   • Filter and analyze specific functions");
    println!("   • Compare different profiling runs");
    println!("   • Generate custom flamegraphs");
    println!("💡 This benchmark demonstrates:");
    println!("   • Sequential vs parallel processing");
    println!("   • Memory allocation patterns");
    println!("   • CPU-bound vs I/O-bound operations");
    println!("   • Different profiling annotation types");
    println!("🎯 Look for hotspots and optimization opportunities!");

    // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // because profile output is only available after that function completes.
    // if let Err(e) = visualization::show_interactive_prompt(
    //     "benchmark",
    //     &ProfileType::Time,
    //     &AnalysisType::Flamechart,
    // ) {
    //     eprintln!("⚠️ Could not show interactive visualization: {e}");
    // }
    smol::block_on(run_analysis(&ProfileType::Time, &AnalysisType::Flamechart));

    // // Interactive visualization: must run AFTER function with `enable_profiling` profiling attribute,
    // // because profile output is only available after that function completes.
    // if let Err(e) = visualization::show_interactive_prompt(
    //     "benchmark",
    //     &ProfileType::Memory,
    //     &AnalysisType::Flamegraph,
    // ) {
    //     eprintln!("⚠️ Could not show interactive visualization: {e}");
    // }
    smol::block_on(run_analysis(
        &ProfileType::Memory,
        &AnalysisType::Flamegraph,
    ));
}
