fn main() {
    println!("Color Comparison (xterm number | owo RGB | xterm RGB | actual xterm color)");
    println!("-------------------------------------------------------------------");

    for color in 0..=255 {
        let owo_rgb = owo_rgb(color);
        let xterm_rgb = xterm_rgb(color);
        println!(
            "#{:03} : \x1b[48;2;{};{};{}m   \x1b[0m : \x1b[48;2;{};{};{}m   \x1b[0m : \x1b[48;5;{}m   \x1b[0m",
            color,
            owo_rgb.0, owo_rgb.1, owo_rgb.2,     // owo RGB in true color
            xterm_rgb.0, xterm_rgb.1, xterm_rgb.2, // xterm RGB in true color
            color                                  // actual xterm color
        );

        // Add breaks between sections
        if color == 15 || color == 231 {
            println!("-------------------------------------------------------------------");
        }
    }
}

fn owo_rgb(color: u8) -> (u8, u8, u8) {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    match color {
        0..=15 => SYSTEM_COLORS[color as usize],
        16..=231 => {
            let color = color - 16;
            let r = STEPS[((color / 36) % 6) as usize];
            let g = STEPS[((color / 6) % 6) as usize];
            let b = STEPS[(color % 6) as usize];
            (r, g, b)
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            (gray, gray, gray)
        }
    }
}

fn xterm_rgb(color: u8) -> (u8, u8, u8) {
    match color {
        0..=15 => SYSTEM_COLORS[color as usize],
        16..=231 => {
            let color = color - 16;
            let r = ((color / 36) % 6) * 51;
            let g = ((color / 6) % 6) * 51;
            let b = (color % 6) * 51;
            (r, g, b)
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            (gray, gray, gray)
        }
    }
}

const SYSTEM_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // black
    (128, 0, 0),     // red
    (0, 128, 0),     // green
    (128, 128, 0),   // yellow
    (0, 0, 128),     // blue
    (128, 0, 128),   // magenta
    (0, 128, 128),   // cyan
    (192, 192, 192), // light gray
    (128, 128, 128), // dark gray
    (255, 0, 0),     // bright red
    (0, 255, 0),     // bright green
    (255, 255, 0),   // bright yellow
    (0, 0, 255),     // bright blue
    (255, 0, 255),   // bright magenta
    (0, 255, 255),   // bright cyan
    (255, 255, 255), // white
];
