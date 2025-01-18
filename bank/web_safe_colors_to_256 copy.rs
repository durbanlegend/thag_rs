use owo_colors::XtermColors;

pub trait Calcs {
    fn rgb_95_40(color: u8) -> (u8, u8, u8);
}

impl Calcs for XtermColors {
    /// Get RGB values for a color number
    fn rgb_95_40(color: u8) -> (u8, u8, u8) {
        const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];

        match color {
            0..=15 => SYSTEM_COLORS[color as usize],
            16..=231 => {
                let color = color as usize - 16;
                let r = STEPS[(color / 36) % 6] as u8;
                let g = STEPS[(color / 6) % 6] as u8;
                let b = STEPS[color % 6] as u8;
                (r, g, b)
            }
            232..=255 => {
                let gray = 8 + (color - 232) * 10;
                (gray, gray, gray)
            } // _ => (0, 0, 0),
        }
    }
}

#[derive(Debug)]
struct WebColor {
    name: String,
    rgb: (u8, u8, u8),
}

impl WebColor {
    fn from_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let name = parts[0].trim().to_string();
            let rgb_str = parts[2].trim();
            if let Some((r, g, b)) = parse_rgb(rgb_str) {
                return Some(WebColor {
                    name,
                    rgb: (r, g, b),
                });
            }
        }
        None
    }
}

fn parse_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let nums: Vec<u8> = s
        .split(',')
        .map(|n| n.trim().parse::<u8>())
        .filter_map(Result::ok)
        .collect();

    if nums.len() == 3 {
        Some((nums[0], nums[1], nums[2]))
    } else {
        None
    }
}

// fn find_closest_color(web_rgb: (u8, u8, u8)) -> u8 {
//     let mut closest_index = 0;
//     let mut min_distance = u32::MAX;

//     for i in 0..=255 {
//         let xterm_rgb = XtermColors::rgb_95_40(i);
//         let distance = color_distance(web_rgb, xterm_rgb);
//         if distance < min_distance {
//             min_distance = distance;
//             closest_index = i;
//         }
//     }
//     closest_index
// }

fn find_closest_color(web_rgb: (u8, u8, u8)) -> u8 {
    let mut closest_index = 0;
    let mut min_distance = u32::MAX;

    for i in 0..=255 {
        let xterm_rgb = XtermColors::rgb_95_40(i);
        let distance = color_distance(web_rgb, xterm_rgb, i < 16);
        if distance < min_distance {
            min_distance = distance;
            closest_index = i;
        }
    }
    closest_index
}

fn convert_color((r, g, b): (u8, u8, u8)) -> u8 {
    // ((r as u32 * 216 + g as u32 * 36 + b as u32 * 6) / 256)
    //     .try_into()
    //     .unwrap()
    r & 0b11100000 | g & 0b11100000 >> 3 | b & 0b11000000 >> 6
}

// fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
//     // Using weighted Euclidean distance for better perceptual matching
//     // Weights based on human perception: R: 0.3, G: 0.59, B: 0.11
//     let dr = (c1.0 as i32 - c2.0 as i32) as f64 * 0.3;
//     let dg = (c1.1 as i32 - c2.1 as i32) as f64 * 0.59;
//     let db = (c1.2 as i32 - c2.2 as i32) as f64 * 0.11;
//     (dr * dr + dg * dg + db * db) as u32
// }

fn color_distance(c1: (u8, u8, u8), c2: (u8, u8, u8), is_system: bool) -> u32 {
    let dr = (c1.0 as i32 - c2.0 as i32) as f64 * 0.3;
    let dg = (c1.1 as i32 - c2.1 as i32) as f64 * 0.59;
    let db = (c1.2 as i32 - c2.2 as i32) as f64 * 0.11;
    let base_distance = (dr * dr + dg * dg + db * db) as u32;

    // Give a slight preference to system colors when they're close matches
    if is_system {
        (base_distance as f64 * 0.9) as u32
    } else {
        base_distance
    }
}

fn main() {
    let content = include_str!("../web_safe_colors.txt");
    let web_colors: Vec<WebColor> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter(|line| line.contains('\t'))
        .filter(|line| !line.starts_with("HTML name"))
        .filter_map(WebColor::from_line)
        .collect();

    println!("Web Color Mappings to 256-color Space:");
    println!("======================================");

    for color in web_colors {
        let closest = find_closest_color(color.rgb);
        let xterm_rgb = XtermColors::rgb_95_40(closest);
        let converted = convert_color(color.rgb);
        let conv_rgb = XtermColors::rgb_95_40(converted);
        println!(
            "{:<20} \x1b[48;2;{};{};{}m   \x1b[0m -> {} {} #{:03} From:({:>3},{:>3},{:>3}) To:({:>3},{:>3},{:>3}), Simple:({:>3},{:>3},{:>3})",
            color.name,
            color.rgb.0, color.rgb.1, color.rgb.2,
            format_color_block(closest),
            format_color_block(converted),
            closest,
            color.rgb.0, color.rgb.1, color.rgb.2,    // From RGB
            xterm_rgb.0, xterm_rgb.1, xterm_rgb.2,    // To RGB
            conv_rgb.0, conv_rgb.1, conv_rgb.2       // Converted RGB
        );
        // println!(
        //     "{:<20} \x1b[48;2;{};{};{}m   \x1b[0m -> {} #{:03} RGB({:>3},{:>3},{:>3})",
        //     color.name,
        //     color.rgb.0,
        //     color.rgb.1,
        //     color.rgb.2,
        //     format_color_block(closest), // Now just passing the color number
        //     closest,
        //     xterm_rgb.0,
        //     xterm_rgb.1,
        //     xterm_rgb.2
        // );
    }
}

fn format_color_block(color_num: u8) -> String {
    match color_num {
        0..=7 => format!("\x1b[4{}m   \x1b[0m", color_num), // Standard colors
        8..=15 => format!("\x1b[10{}m   \x1b[0m", color_num - 8), // Bright colors
        _ => format!("\x1b[48;5;{}m   \x1b[0m", color_num), // 256 colors
    }
    // match color_num {
    //     0..=7 => format!("\x1b[40m   \x1b[0m", color_num), // Standard colors
    //     8..=15 => format!("\x1b[100m   \x1b[0m", color_num - 8), // Bright colors
    //     _ => format!("\x1b[48;5;{}m   \x1b[0m", color_num), // 256 colors
    // }
}

// RGB values for system colors (0-15)
const SYSTEM_COLORS: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00), // Black
    (0x80, 0x00, 0x00), // Red
    (0x00, 0x80, 0x00), // Green
    (0x80, 0x80, 0x00), // Yellow
    (0x00, 0x00, 0x80), // Blue
    (0x80, 0x00, 0x80), // Magenta
    (0x00, 0x80, 0x80), // Cyan
    (0xc0, 0xc0, 0xc0), // LightGray
    (0x80, 0x80, 0x80), // DarkGray
    (0xff, 0x00, 0x00), // BrightRed
    (0x00, 0xff, 0x00), // BrightGreen
    (0xff, 0xff, 0x00), // BrightYellow
    (0x00, 0x00, 0xff), // BrightBlue
    (0xff, 0x00, 0xff), // BrightMagenta
    (0x00, 0xff, 0xff), // BrightCyan
    (0xff, 0xff, 0xff), // White
];
