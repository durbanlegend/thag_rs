/// Prototype of styling with GruvBox Light theme colours.
//# Purpose: Investigate incorporating popular themes into styling.
//# Categories: technique
fn print_styled(style: &str, text: &str) {
    match style {
        // Headers and Structure
        "heading1" => println!("\x1b[1;38;5;124m{}\x1b[0m", text), // Bold Red
        "heading2" => println!("\x1b[1;38;5;106m{}\x1b[0m", text), // Bold Green
        "subheading" => println!("\x1b[1;38;5;172m{}\x1b[0m", text), // Bold Yellow
        "section" => println!("\x1b[4;38;5;66m{}\x1b[0m", text),   // Underlined Blue

        // Alerts and Status
        "error" => println!("\x1b[1;38;5;167m{}\x1b[0m", text), // Bold Bright Red
        "warning" => println!("\x1b[38;5;214m{}\x1b[0m", text), // Bright Yellow
        "success" => println!("\x1b[38;5;142m{}\x1b[0m", text), // Bright Green
        "info" => println!("\x1b[38;5;109m{}\x1b[0m", text),    // Bright Blue

        // Emphasis Levels
        "emphasis" => println!("\x1b[1;38;5;132m{}\x1b[0m", text), // Bold Purple
        "bright" => println!("\x1b[38;5;108m{}\x1b[0m", text),     // Bright Aqua
        "normal" => println!("\x1b[38;5;237m{}\x1b[0m", text),     // Normal Gray
        "subtle" => println!("\x1b[38;5;245m{}\x1b[0m", text),     // Light Gray
        "ghost" => println!("\x1b[2;38;5;245m{}\x1b[0m", text),    // Dim Gray

        // Debug and Development
        "debug" => println!("\x1b[3;38;5;245m{}\x1b[0m", text), // Italic Gray
        "trace" => println!("\x1b[2;38;5;243m{}\x1b[0m", text), // Dim Darker Gray
        "diagnostic" => println!("\x1b[3;38;5;66m{}\x1b[0m", text), // Italic Blue

        // Code and Data
        "code" => println!("\x1b[38;5;106m{}\x1b[0m", text), // Green
        "variable" => println!("\x1b[3;38;5;172m{}\x1b[0m", text), // Italic Yellow
        "value" => println!("\x1b[38;5;124m{}\x1b[0m", text), // Red

        _ => println!("{}", text),
    }
}

fn main() {
    println!("Gruvbox Light Theme Extended Styles:");
    println!("---------------------------------------");

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
