/*[toml]
[dependencies]
crossterm = "*"
ctrlc = "3.3"
reedline = "0.32.0"
*/

use reedline::{DefaultPromptEditMode, DefaultTerminal, Reedline, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize Reedline
    let mut rl = Reedline::create().unwrap();
    rl.set_prompt(">> ");
    rl.set_prompt_edit_mode(DefaultPromptEditMode::Emacs);

    // Set up Ctrl-C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // REPL loop
    while running.load(Ordering::SeqCst) {
        if let Some(input) = rl.read_line().unwrap() {
            if input.trim() == "quit" {
                // Send Ctrl-D signal
                send_ctrl_d();
                break;
            } else {
                println!("You entered: {}", input);
            }
        }
    }

    println!("Exiting...");
}

fn send_ctrl_d() {
    // Sleep for a short duration to ensure Ctrl-C is registered before sending Ctrl-D
    thread::sleep(Duration::from_millis(10));

    // Send Ctrl-D signal
    ctrlc::kill(Signal::SIGINT).expect("Error sending Ctrl-D signal");
}
