fn print_styled(style: &str, text: &str) {
    match style {
        "heading1" => println!("\x1b[1;38;5;124m{}\x1b[0m", text), // Bold, Red
        "heading2" => println!("\x1b[1;38;5;106m{}\x1b[0m", text), // Bold, Green
        "heading3" => println!("\x1b[1;38;5;172m{}\x1b[0m", text), // Bold, Yellow
        "error" => println!("\x1b[38;5;167m{}\x1b[0m", text),      // Bright Red
        "warning" => println!("\x1b[38;5;214m{}\x1b[0m", text),    // Bright Yellow
        "info" => println!("\x1b[38;5;109m{}\x1b[0m", text),       // Bright Blue
        "comment" => println!("\x1b[3;38;5;245m{}\x1b[0m", text),  // Italic, Bright Gray
        "string" => println!("\x1b[38;5;106m{}\x1b[0m", text),     // Green
        "keyword" => println!("\x1b[1;38;5;132m{}\x1b[0m", text),  // Bold, Purple
        _ => println!("{}", text),
    }
}

fn main() {
    println!("Gruvbox Light Hard Theme Styles:");
    println!("-------------------------------");

    print_styled("heading1", "Heading 1");
    print_styled("heading2", "Heading 2");
    print_styled("heading3", "Heading 3");
    print_styled("error", "Error: Something went wrong!");
    print_styled("warning", "Warning: Proceed with caution");
    print_styled("info", "Info: Here's some information");
    print_styled("comment", "// This is a comment");
    print_styled("string", "\"This is a string literal\"");
    print_styled("keyword", "let const function if else");
}
