use itertools::Itertools;
use owo_colors::XtermColors;

//: Map and visually test conversion of web safe colours to 256 colours, //: using the `owo-colors` crate colour names and mappings.
//# Purpose: Work out and test colour conversion.
//# Categories: demo, reference, testing
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

fn find_closest_color(web_rgb: (u8, u8, u8)) -> u8 {
    let mut closest_index = 0;
    let mut min_distance = u32::MAX;

    for i in 0..=255 {
        let owo_rgb = XtermColors::rgb_95_40(i);
        let distance = color_distance(web_rgb, owo_rgb, i < 16);
        if distance < min_distance {
            min_distance = distance;
            closest_index = i;
        }
    }
    closest_index
}

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
        .sorted_by(|a, b| {
            a.rgb
                .0
                .cmp(&b.rgb.0) // First sort by R
                .then_with(|| a.rgb.1.cmp(&b.rgb.1)) // Then by G
                .then_with(|| a.rgb.2.cmp(&b.rgb.2)) // Then by B
        })
        .collect();

    println!("Web Color Mappings to 256-color Space:");
    println!("======================================");

    println!(
        "Name:                Web    owo ->RGB Formatted owo RGB                   owo# From:   Web RGB    To: owo Xtrm RGB"
    );
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────────────");
    for web_color in web_colors {
        let owo = find_closest_color(web_color.rgb);
        let owo_block = format_color_block(owo);
        let owo_rgb = XtermColors::rgb_95_40(owo);
        //      Name web color block                o_256 o_rgb                 Owo formatted
        println!(
             "{:<20} \x1b[48;2;{};{};{}m   \x1b[0m -> {}  \x1b[48;2;{};{};{}m   \x1b[0m  {:<35} #{:03} From:({:>3},{:>3},{:>3}) To:({:>3},{:>3},{:>3})",
             web_color.name,    // Name
             web_color.rgb.0, web_color.rgb.1, web_color.rgb.2, // Web colour block
             owo_block, // owo 256 color block
             owo_rgb.0, owo_rgb.1, owo_rgb.2, // owo rgb block
             format!("{owo_block:?}"),
             owo,   // #0-255
             web_color.rgb.0, web_color.rgb.1, web_color.rgb.2,    // From RGB
             owo_rgb.0, owo_rgb.1, owo_rgb.2     // To RGB
         );
    }
}

fn format_color_block(color_num: u8) -> String {
    match color_num {
        0..=7 => format!("\x1b[3{}m   \x1b[0m", color_num), // Standard colors
        8..=15 => format!("\x1b[9{}m   \x1b[0m", color_num - 8), // Bright colors
        _ => format!("\x1b[48;5;{}m   \x1b[0m", color_num), // 256 colors
    }
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
