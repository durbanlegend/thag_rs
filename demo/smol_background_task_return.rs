/// ChatGPT-generated example of running a single task in the background.
//# Purpose: Demo.
//# Categories: crates. demo
use anyhow::Result;
use smol::Timer;
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

/// Async background work that takes arguments and can fail.
/// Here we simulate "slow I/O" by reading a file *off the async executor*
/// using `smol::unblock`, then do a trivial computation.
///
/// Any I/O error propagates via `?`.
async fn async_bg_work(path: PathBuf, multiplier: u64) -> Result<u64> {
    // Run blocking std::fs I/O on Smol's blocking pool.
    let data = smol::unblock(move || std::fs::read(path)).await?;
    // Trivial "work": scale the file length.
    Ok((data.len() as u64) * multiplier)
}

/// The async top-level that orchestrates foreground progress + background work.
async fn run_demo() -> Result<()> {
    // Choose something to read; adjust as needed.
    let path = PathBuf::from("Cargo.toml");
    let multiplier = 2;

    // Spawn the background future. We move the arguments into the async fn.
    let bg_task = smol::spawn(async_bg_work(path, multiplier));

    // Foreground "main thread" progress display.
    for i in 1..=5 {
        print!("Foreground step {i}/5\r");
        io::stdout().flush()?; // force display
        Timer::after(Duration::from_millis(200)).await;
    }
    println!(); // newline after the carriage-return updates
    io::stdout().flush()?; // final flush before join

    // Join: await the background result. Use `?` to bubble up any error.
    let value = bg_task.await?;
    println!("Background computed value: {value}");

    Ok(())
}

fn main() -> Result<()> {
    // Drive the async code to completion and surface any error.
    smol::block_on(run_demo())
}
