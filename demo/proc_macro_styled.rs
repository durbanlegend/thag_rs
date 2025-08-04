/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto" }
*/

/// Testing the `styled` proc macro
//# Purpose: Test the proof of concept and potentially the implementation.
use thag_demo_proc_macros::styled;
use thag_styling::{cprtln, Role, Style, V};

fn main() {
    let name = "Error";
    println!("{}", styled!(bold, => name));

    cprtln!(
        Style::for_role(Role::Heading2),
        "error={}",
        styled!(bold, italic, underlined, => name)
    );
}
