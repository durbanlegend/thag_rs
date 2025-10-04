/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto" }
*/
/// This program exists to demonstrate the `thag_common` `auto_help` functionality.
/// Invoking it with the argument `--help/-h` will display the doc comments as help.
/// An optional `//# Purpose: ` line may be included to form the top-level help summary.
/// An optional `//# Categories:` line may be used to list comma-separated categories
/// to be shown at the bottom of the help screen.
//# Purpose: This is the optional `//# Purpose: ` line that becomes the help summary.
//# Categories: demo, testing
use thag_common::{auto_help, help_system::check_help_and_exit};
fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    let other = "World 🌍";
    println!("Hello, {other}!");
}
