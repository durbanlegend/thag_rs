/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

/// Testing the `styled` proc macro
//# Purpose: Test the proof of concept and potentially the implementation.
use thag_styling::{cprtln, styled, AnsiStyleExt, Role, Style};

fn main() {
    let name = "Should be bold";
    // println!("{}", styled!((name), bold);
    println!("Should be normal {} Should be normal", styled!(name, bold));

    cprtln!(
        Style::for_role(Role::Success),
        "error={}, now for some boilerplate",
        styled!(name, italic, underline)
    );
}
