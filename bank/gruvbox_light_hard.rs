fn print_color(color_code: u8, description: &str) {
    println!("\x1b[48;5;{}m     \x1b[0m {}", color_code, description);
}

fn main() {
    println!("Gruvbox Light Hard Theme (xterm-256 colors):");
    println!("-------------------------------------------");

    // Background and Foreground
    print_color(230, "bg0     (Background)");
    print_color(235, "fg0     (Foreground)");

    // Basic Colors
    print_color(124, "red");
    print_color(106, "green");
    print_color(172, "yellow");
    print_color(66, "blue");
    print_color(132, "purple");
    print_color(72, "aqua");
    print_color(237, "gray");

    // Bright Colors
    print_color(167, "bright red");
    print_color(142, "bright green");
    print_color(214, "bright yellow");
    print_color(109, "bright blue");
    print_color(175, "bright purple");
    print_color(108, "bright aqua");
    print_color(245, "bright gray");
}
