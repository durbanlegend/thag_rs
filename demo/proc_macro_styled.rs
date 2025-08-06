/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_styling = { version = "0.2, thag-auto" }
*/

/// Testing the `styled` proc macro with `ansi_styling_support`
//# Purpose: Test the styled! macro with generated ANSI styling support.
//# Categories: ansi, color, demo, macros, proc_macros, styling, terminal
use thag_proc_macros::{ansi_styling_support, styled};
use thag_styling::{cprtln, Role};

// Generate ANSI styling support - no need to import AnsiStyleExt!
ansi_styling_support! {}

fn main() {
    let name = "Should be bold";
    println!("Should be normal {} Should be normal", styled!(name, bold));

    cprtln!(
        Role::Success,
        "error={}, now for some boilerplate",
        styled!(name, italic, underline)
    );

    // Demonstrate more styling options now that we have the full implementation
    println!("{}", styled!("Red and bold", fg = Red, bold));
    println!("{}", styled!("Blue underlined", fg = Blue, underline));
    println!(
        "{}",
        styled!("Green italic reversed", fg = Green, italic, reversed)
    );

    // Show that we can also use the trait methods directly
    println!("{}", "Direct trait usage".style().fg(Color::Magenta).bold());
}
