/// ChatGPT-generated example of running a single task in the background.
//# Purpose: Demo.
//# Categories: crates, demo
use smol::{self, Timer};
use std::io::{self, Write};
use std::time::Duration;

// Some blocking work we don't want to run on the async reactor thread.
// Pretend it's CPU-heavy or does blocking I/O.
fn slow_blocking_compute() -> u64 {
    // Simulate ~1.5s of work
    std::thread::sleep(Duration::from_millis(1500));
    42
}

fn main() {
    // Drive async code to completion on the current thread.
    smol::block_on(async {
        // Kick off the blocking work *immediately* on Smol's blocking pool.
        // Returns a Task<u64> that we can await later.
        let bg_task = smol::unblock(slow_blocking_compute);

        // While the blocking work runs elsewhere, show progress in "main thread".
        // (We're still inside an async context, but doing synchronous prints here
        // is fine for a short demo. For a TUI you'd likely do better coordination.)
        for i in 1..=5 {
            print!("Working... step {i}/5\r");
            // Explicit flush so the user sees the update now.
            let _ = io::stdout().flush();
            // Simulate other async work; yield briefly without blocking the executor.
            Timer::after(Duration::from_millis(250)).await;
        }

        // Move to a clean line, flush final display before we join.
        println!("\nWaiting for background result...");
        let _ = io::stdout().flush();

        // Join the background task (await it) *after* we've finished displaying.
        match bg_task.await {
            result => println!("Background task completed with value: {result}"),
        }
    });
}
