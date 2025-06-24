/*[toml]
[dependencies]
thag_rs = { version = "0.2", path = "../..", default-features = false, features = ["core"] }
*/

/// Demo tool to showcase the lightweight help system.
///
/// This tool demonstrates how to use the built-in help system for thag tools.
/// It shows how help information is extracted from comments and displayed consistently.
//# Purpose: Demonstrate the lightweight help system for thag tools
//# Categories: demo, tools
//# Usage: thag_demo_help [--help|-h]
use std::env;
use thag_rs::{auto_help, help_system::check_help_and_exit};

fn main() {
    // Initialize help system from source comments automatically
    let help = auto_help!("thag_demo_help");

    // Check if help was requested and exit if so
    check_help_and_exit(&help);

    // Main program logic
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        println!("You provided {} argument(s):", args.len() - 1);
        for (i, arg) in args.iter().skip(1).enumerate() {
            println!("  {}: {}", i + 1, arg);
        }
    } else {
        println!("This is a demo tool showcasing the thag help system!");
        println!("Try running with --help to see the help information.");
        println!("You can also pass some arguments to see them echoed back.");
    }

    println!("\nThis help information was automatically extracted from source comments.");
}
