/// Prototype of styling with GruvBox Light Hard theme colours.
//# Purpose: Investigate incorporating popular themes into styling.
//# Categories: technique
fn print_styled(style: &str, text: &str) {
    match style {
        // Headers and Structure
        "heading1" => println!("\x1b[1;38;5;124m{}\x1b[0m", text), // Bold Red
        "heading2" => println!("\x1b[1;38;5;100m{}\x1b[0m", text), // Bold Green
        "subheading" => println!("\x1b[1;38;5;172m{}\x1b[0m", text), // Bold Orange/Yellow
        "section" => println!("\x1b[4;38;5;124m{}\x1b[0m", text),  // Underlined Red

        // Alerts and Status
        "error" => println!("\x1b[38;5;160m{}\x1b[0m", text), // Bright Red
        "warning" => println!("\x1b[38;5;214m{}\x1b[0m", text), // Bright Yellow
        "success" => println!("\x1b[38;5;142m{}\x1b[0m", text), // Bright Green
        "info" => println!("\x1b[38;5;66m{}\x1b[0m", text),   // Bright Blue

        // Emphasis Levels
        "emphasis" => println!("\x1b[1;38;5;126m{}\x1b[0m", text), // Bold Purple
        "bright" => println!("\x1b[38;5;72m{}\x1b[0m", text),      // Bright Aqua
        "normal" => println!("\x1b[38;5;239m{}\x1b[0m", text),     // Dark Gray
        "subtle" => println!("\x1b[38;5;243m{}\x1b[0m", text),     // Medium Gray
        "ghost" => println!("\x1b[38;5;245m{}\x1b[0m", text),      // Light Gray

        // Debug and Development
        "debug" => println!("\x1b[3;38;5;166m{}\x1b[0m", text), // Italic Orange
        "trace" => println!("\x1b[38;5;246m{}\x1b[0m", text),   // Gray

        _ => println!("{}", text),
    }
}

fn main() {
    println!("Gruvbox Light Hard Theme Styles:");
    println!("-----------------------------");

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
}
