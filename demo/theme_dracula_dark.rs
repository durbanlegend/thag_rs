/// Prototype of styling with Dracula theme colours.
//# Purpose: Investigate incorporating popular themes into styling.
//# Categories: technique
fn print_styled(style: &str, text: &str) {
    match style {
        // Headers and Structure
        "heading1" => println!("\x1b[1;38;5;212m{}\x1b[0m", text), // Bold Pink
        "heading2" => println!("\x1b[1;38;5;141m{}\x1b[0m", text), // Bold Purple
        "subheading" => println!("\x1b[1;38;5;117m{}\x1b[0m", text), // Bold Cyan
        "section" => println!("\x1b[4;38;5;141m{}\x1b[0m", text),  // Underlined Purple

        // Alerts and Status
        "error" => println!("\x1b[38;5;203m{}\x1b[0m", text), // Red
        "warning" => println!("\x1b[38;5;228m{}\x1b[0m", text), // Yellow
        "success" => println!("\x1b[38;5;84m{}\x1b[0m", text), // Green
        "info" => println!("\x1b[38;5;117m{}\x1b[0m", text),  // Cyan

        // Emphasis Levels
        "emphasis" => println!("\x1b[1;38;5;141m{}\x1b[0m", text), // Bold Purple
        "bright" => println!("\x1b[38;5;117m{}\x1b[0m", text),     // Cyan
        "normal" => println!("\x1b[38;5;253m{}\x1b[0m", text),     // Light Gray
        "subtle" => println!("\x1b[38;5;245m{}\x1b[0m", text),     // Medium Gray
        "ghost" => println!("\x1b[2;38;5;244m{}\x1b[0m", text),    // Dim Light Gray (brightened)

        // Debug and Development
        "debug" => println!("\x1b[3;38;5;245m{}\x1b[0m", text), // Italic Medium Gray
        "trace" => println!("\x1b[2;38;5;244m{}\x1b[0m", text), // Dim Light Gray (brightened)
        "diagnostic" => println!("\x1b[3;38;5;117m{}\x1b[0m", text), // Italic Cyan

        // Code and Data
        "code" => println!("\x1b[38;5;84m{}\x1b[0m", text), // Green
        "variable" => println!("\x1b[3;38;5;215m{}\x1b[0m", text), // Italic Orange
        "value" => println!("\x1b[38;5;228m{}\x1b[0m", text), // Yellow

        _ => println!("{}", text),
    }
}

fn main() {
    println!("Dracula Theme Styles:");
    println!("--------------------");

    // Headers
    print_styled("heading1", "Main Heading");
    print_styled("heading2", "Secondary Heading");
    print_styled("subheading", "Subheading");
    print_styled("section", "Section Divider");
    println!();

    // Alerts
    print_styled("error", "Error: Critical failure detected");
    print_styled("warning", "Warning: Proceed with caution");
    print_styled("success", "Success: Operation completed");
    print_styled("info", "Info: Standard information");
    println!();

    // Emphasis Levels
    print_styled("emphasis", "Emphasized important text");
    print_styled("bright", "Bright highlighted text");
    print_styled("normal", "Normal regular text");
    print_styled("subtle", "Subtle background info");
    print_styled("ghost", "Ghost text (de-emphasized)");
    println!();

    // Debug
    print_styled("debug", "Debug: Diagnostic information");
    print_styled("trace", "Trace: Detailed execution path");
    print_styled("diagnostic", "Diagnostic: System state");
    println!();

    // Code
    print_styled("code", "let x = 42;");
    print_styled("variable", "x");
    print_styled("value", "42");
}
