/// ChatGPT-generated example of running a single task in the background.
//# Purpose: Demo.
//# Categories: crates, demo
use smol::{self, Timer};
use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

// Assume AnalysisType is defined somewhere
#[derive(Debug)]
struct AnalysisType;

// Your real function
fn generate_and_show_memory_flamegraph(
    demo_name: &str,
    analysis_type: AnalysisType,
    files: Vec<PathBuf>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Simulate blocking work
    std::thread::sleep(Duration::from_millis(1000));
    println!(
        "\n[Background] Generating flamegraph for demo: {} ({:?})",
        demo_name, analysis_type
    );
    println!("[Background] Files: {:?}", files);
    Ok(())
}

async fn run_demo() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let demo_name = "example_demo";
    let analysis_type = AnalysisType;
    // let analysis_type_lower = "example".to_string();
    let files = vec![PathBuf::from("Cargo.toml")];

    // Spawn the blocking function onto Smol's blocking pool.
    // We move in all owned arguments except references (we can clone demo_name).
    let demo_name_owned = demo_name.to_string();
    // let analysis_type_ref = &analysis_type;
    let files_clone = files.clone();

    let bg_task = smol::unblock(move || {
        generate_and_show_memory_flamegraph(&demo_name_owned, analysis_type, files_clone)
    });

    // Foreground progress
    for i in 1..=5 {
        print!("Foreground step {i}/5\r");
        io::stdout().flush()?;
        Timer::after(Duration::from_millis(200)).await;
    }
    println!();
    io::stdout().flush()?;

    // Join the background task, propagate error if any
    bg_task.await?;

    println!("Background flamegraph generation complete!");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    smol::block_on(run_demo())
}
