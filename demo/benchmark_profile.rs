/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["time_profiling"] }
# thag_profiler = { version = "0.1", features = ["time_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["time_profiling"] }

[profile.release]
# debug-assertions = true
debug = true
strip = false
*/

/// ChagtGPT-generated profiling synchronous time profiling benchmark: `thag_profiler` implementation`.
/// See `demo/benchmark*.rs` for base code and `firestorm` implementation.
///
//# Purpose: For checking and comparison of profiling tools
//# Categories: benchmark, profiling
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use regex::Regex;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::Instant;
use thag_profiler::{enable_profiling, profiled};

const WORK_SIZE: usize = 500_000; // Increase this to scale up

#[profiled]
fn generate_logs() -> Vec<String> {
    let mut rng = StdRng::seed_from_u64(42);
    let actions = ["login", "logout", "click", "scroll", "type"];
    (0..WORK_SIZE)
        .map(|_i| {
            let user = rng.random_range(1..100);
            let action = actions[rng.random_range(0..actions.len())];
            let time = rng.random_range(1..10_000);
            format!("INFO - user={user} action={action} time={time}")
        })
        .collect()
}

#[profiled]
fn write_logs_to_file(filename: &str, logs: &[String]) {
    let mut file = File::create(filename).unwrap();
    for line in logs {
        writeln!(file, "{line}").unwrap();
    }
}

#[profiled]
fn read_and_parse_logs(filename: &str) -> Vec<(u32, String, u64)> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let re = Regex::new(r"user=(\d+)\s+action=(\w+)\s+time=(\d+)").unwrap();

    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let caps = re.captures(&line)?;
            Some((
                caps[1].parse().ok()?,
                caps[2].to_string(),
                caps[3].parse().ok()?,
            ))
        })
        .collect()
}

#[profiled]
fn analyze(logs: &[(u32, String, u64)]) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for (_, action, time) in logs {
        *map.entry(action.clone()).or_insert(0) += time;
    }
    map
}

#[profiled]
fn heavy_hashing_work(data: &[(u32, String, u64)]) -> Vec<String> {
    data.par_iter()
        .map(|(user, action, time)| {
            let mut hasher = Sha256::new();
            hasher.update(format!("{user}:{action}:{time}"));
            format!("{:x}", hasher.finalize())
        })
        .collect()
}

#[profiled]
fn simulate_output(data: &HashMap<String, u64>, hashes: &[String]) {
    let summary = json!({
        "totals": data,
        "samples": &hashes[..std::cmp::min(10, hashes.len())],
    });
    println!("{summary}");
}

#[enable_profiling(time)]
fn main() {
    let start = Instant::now();

    let logs = generate_logs();
    write_logs_to_file("benchmark.log", &logs);
    let parsed = read_and_parse_logs("benchmark.log");
    let summary = analyze(&parsed);
    let hashes = heavy_hashing_work(&parsed);
    simulate_output(&summary, &hashes);

    let elapsed = start.elapsed();
    eprintln!("Elapsed: {elapsed:.2?}");
}
