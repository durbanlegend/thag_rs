/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
*/

/// Demonstrates using the ansi_styling_support! macro to generate ANSI styling capabilities
///
/// E.g. `thag demo/ansi_styling_support.rs`
//# Purpose: Demonstrate the ansi_styling_support! macro for standalone ANSI styling.
//# Categories: ansi, color, demo, macros, proc_macros, styling, terminal
use thag_proc_macros::{ansi_styling_support, styled};

// Generate all the styling support code
ansi_styling_support! {}

fn main() {
    println!("{}", "Bold Red".style().bold().fg(Color::Red));
    println!("{}", styled!("Underlined Green", fg = Green, underline));
    println!("{}", "Italic Blue".style().italic().fg(Color::Blue));
    println!(
        "{}",
        "Bold underlined magenta"
            .style()
            .bold()
            .underline()
            .fg(Color::Magenta)
    );
    println!(
        "{}",
        styled!(
            "Italic, Underlined, Yellow, Reversed",
            italic,
            underline,
            fg = Yellow,
            reversed
        )
    );
    println!("{}", "Normal text".style());

    let name = "Success";
    println!("{}", styled!(name, bold, fg = Green));
    println!(
        "{}",
        styled!(format!("User: {}", "alice"), fg = Blue, underline)
    );

    // Demonstrate embedding styled content
    let outer = "outer ".style().fg(Color::Red);
    let inner = "inner".style().fg(Color::Green);
    println!("{}{} world", outer, outer.embed(inner));

    // Show different color combinations
    println!("\nColor showcase:");
    println!("{}", styled!("Black", fg = Black, bold));
    println!("{}", styled!("Red", fg = Red, bold));
    println!("{}", styled!("Green", fg = Green, bold));
    println!("{}", styled!("Yellow", fg = Yellow, bold));
    println!("{}", styled!("Blue", fg = Blue, bold));
    println!("{}", styled!("Magenta", fg = Magenta, bold));
    println!("{}", styled!("Cyan", fg = Cyan, bold));
    println!("{}", styled!("White", fg = White, bold));

    println!("\nEffect showcase:");
    println!("{}", styled!("Bold text", bold));
    println!("{}", styled!("Italic text", italic));
    println!("{}", styled!("Underlined text", underline));
    println!("{}", styled!("Reversed text", reversed));
    println!(
        "{}",
        styled!("Combined effects", bold, italic, underline, fg = Cyan)
    );
}
