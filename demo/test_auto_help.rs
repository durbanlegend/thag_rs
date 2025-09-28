/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto" }
*/
/// Demo / test of `thag_common`'s `auto_help` system. This is the first doc comment line.
/// This is the second doc comment line.
//# Purpose: Demo Hello World as a program. This is the `//# Purpose: ` line.
//# Categories: demo, testing
use thag_common::{auto_help, help_system::check_help_and_exit};
fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    let other = "World 🌍";
    println!("Hello, {other}!");
}
