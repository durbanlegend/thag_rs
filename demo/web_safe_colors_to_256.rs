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
    let content = r#"HTML name	R G B Hex	Decimal
Pink colors
MediumVioletRed	C71585	199, 21, 133
DeepPink	FF1493	255, 20, 147
PaleVioletRed	DB7093	219, 112, 147
HotPink	FF69B4	255, 105, 180
LightPink	FFB6C1	255, 182, 193
Pink	FFC0CB	255, 192, 203
Red colors
DarkRed	8B0000	139, 0, 0
Red	FF0000	255, 0, 0
Firebrick	B22222	178, 34, 34
Crimson	DC143C	220, 20, 60
IndianRed	CD5C5C	205, 92, 92
LightCoral	F08080	240, 128, 128
Salmon	FA8072	250, 128, 114
DarkSalmon	E9967A	233, 150, 122
LightSalmon	FFA07A	255, 160, 122
Orange colors
OrangeRed	FF4500	255, 69, 0
Tomato	FF6347	255, 99, 71
DarkOrange	FF8C00	255, 140, 0
Coral	FF7F50	255, 127, 80
Orange	FFA500	255, 165, 0
Yellow colors
DarkKhaki	BDB76B	189, 183, 107
Gold	FFD700	255, 215, 0
Khaki	F0E68C	240, 230, 140
PeachPuff	FFDAB9	255, 218, 185
Yellow	FFFF00	255, 255, 0
PaleGoldenrod	EEE8AA	238, 232, 170
Moccasin	FFE4B5	255, 228, 181
PapayaWhip	FFEFD5	255, 239, 213
LightGoldenrodYellow	FAFAD2	250, 250, 210
LemonChiffon	FFFACD	255, 250, 205
LightYellow	FFFFE0	255, 255, 224
Brown colors
Maroon	800000	128, 0, 0
Brown	A52A2A	165, 42, 42
SaddleBrown	8B4513	139, 69, 19
Sienna	A0522D	160, 82, 45
Chocolate	D2691E	210, 105, 30
DarkGoldenrod	B8860B	184, 134, 11
Peru	CD853F	205, 133, 63
RosyBrown	BC8F8F	188, 143, 143
Goldenrod	DAA520	218, 165, 32
SandyBrown	F4A460	244, 164, 96
Tan	D2B48C	210, 180, 140
Burlywood	DEB887	222, 184, 135
Wheat	F5DEB3	245, 222, 179
NavajoWhite	FFDEAD	255, 222, 173
Bisque	FFE4C4	255, 228, 196
BlanchedAlmond	FFEBCD	255, 235, 205
Cornsilk	FFF8DC	255, 248, 220
Purple, violet, and magenta colors
Indigo	4B0082	75, 0, 130
Purple	800080	128, 0, 128
DarkMagenta	8B008B	139, 0, 139
DarkViolet	9400D3	148, 0, 211
DarkSlateBlue	483D8B	72, 61, 139
BlueViolet	8A2BE2	138, 43, 226
DarkOrchid	9932CC	153, 50, 204
Fuchsia	FF00FF	255, 0, 255
Magenta	FF00FF	255, 0, 255
SlateBlue	6A5ACD	106, 90, 205
MediumSlateBlue	7B68EE	123, 104, 238
MediumOrchid	BA55D3	186, 85, 211
MediumPurple	9370DB	147, 112, 219
Orchid	DA70D6	218, 112, 214
Violet	EE82EE	238, 130, 238
Plum	DDA0DD	221, 160, 221
Thistle	D8BFD8	216, 191, 216
Lavender	E6E6FA	230, 230, 250
Blue colors
MidnightBlue	191970	25, 25, 112
Navy	000080	0, 0, 128
DarkBlue	00008B	0, 0, 139
MediumBlue	0000CD	0, 0, 205
Blue	0000FF	0, 0, 255
RoyalBlue	4169E1	65, 105, 225
SteelBlue	4682B4	70, 130, 180
DodgerBlue	1E90FF	30, 144, 255
DeepSkyBlue	00BFFF	0, 191, 255
CornflowerBlue	6495ED	100, 149, 237
SkyBlue	87CEEB	135, 206, 235
LightSkyBlue	87CEFA	135, 206, 250
LightSteelBlue	B0C4DE	176, 196, 222
LightBlue	ADD8E6	173, 216, 230
PowderBlue	B0E0E6	176, 224, 230
Cyan colors
Teal	008080	0, 128, 128
DarkCyan	008B8B	0, 139, 139
LightSeaGreen	20B2AA	32, 178, 170
CadetBlue	5F9EA0	95, 158, 160
DarkTurquoise	00CED1	0, 206, 209
MediumTurquoise	48D1CC	72, 209, 204
Turquoise	40E0D0	64, 224, 208
Aqua	00FFFF	0, 255, 255
Cyan	00FFFF	0, 255, 255
Aquamarine	7FFFD4	127, 255, 212
PaleTurquoise	AFEEEE	175, 238, 238
LightCyan	E0FFFF	224, 255, 255
Green colors
DarkGreen	006400	0, 100, 0
Green	008000	0, 128, 0
DarkOliveGreen	556B2F	85, 107, 47
ForestGreen	228B22	34, 139, 34
SeaGreen	2E8B57	46, 139, 87
Olive	808000	128, 128, 0
OliveDrab	6B8E23	107, 142, 35
MediumSeaGreen	3CB371	60, 179, 113
LimeGreen	32CD32	50, 205, 50
Lime	00FF00	0, 255, 0
SpringGreen	00FF7F	0, 255, 127
MediumSpringGreen	00FA9A	0, 250, 154
DarkSeaGreen	8FBC8F	143, 188, 143
MediumAquamarine	66CDAA	102, 205, 170
YellowGreen	9ACD32	154, 205, 50
LawnGreen	7CFC00	124, 252, 0
Chartreuse	7FFF00	127, 255, 0
LightGreen	90EE90	144, 238, 144
GreenYellow	ADFF2F	173, 255, 47
PaleGreen	98FB98	152, 251, 152
White colors
MistyRose	FFE4E1	255, 228, 225
AntiqueWhite	FAEBD7	250, 235, 215
Linen	FAF0E6	250, 240, 230
Beige	F5F5DC	245, 245, 220
WhiteSmoke	F5F5F5	245, 245, 245
LavenderBlush	FFF0F5	255, 240, 245
OldLace	FDF5E6	253, 245, 230
AliceBlue	F0F8FF	240, 248, 255
Seashell	FFF5EE	255, 245, 238
GhostWhite	F8F8FF	248, 248, 255
Honeydew	F0FFF0	240, 255, 240
FloralWhite	FFFAF0	255, 250, 240
Azure	F0FFFF	240, 255, 255
MintCream	F5FFFA	245, 255, 250
Snow	FFFAFA	255, 250, 250
Ivory	FFFFF0	255, 255, 240
White	FFFFFF	255, 255, 255
Gray and black colors
Black	000000	0, 0, 0
DarkSlateGray	2F4F4F	47, 79, 79
DimGray	696969	105, 105, 105
SlateGray	708090	112, 128, 144
Gray	808080	128, 128, 128
LightSlateGray	778899	119, 136, 153
DarkGray	A9A9A9	169, 169, 169
Silver	C0C0C0	192, 192, 192
LightGray	D3D3D3	211, 211, 211
Gainsboro	DCDCDC	220, 220, 220

Source: https://en.wikipedia.org/wiki/Web_colors
"#;

    let web_colors: Vec<WebColor> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter(|line| line.contains('\t'))
        .filter(|line| !line.starts_with("HTML name"))
        .filter(|line| !line.starts_with("Source:"))
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
        "Name:                Web    owo ->RGB  ----- Formatted owo RGB ------    owo#   From:   web RGB    to: owo xtrm RGB"
    );
    println!("──────────────────────────────────────────────────────────────────────────────────────────────────────────────────");
    for web_color in web_colors {
        let owo = find_closest_color(web_color.rgb);
        let owo_block = format_color_block(owo);
        let owo_rgb = XtermColors::rgb_95_40(owo);
        //      Name web color block                o_256 o_rgb                 Owo formatted
        println!(
             "{:<20} \x1b[48;2;{};{};{}m   \x1b[0m -> {}   \x1b[48;2;{};{};{}m   \x1b[0m  {:<33} #{:03}   From:({:>3},{:>3},{:>3}) to:({:>3},{:>3},{:>3})",
             web_color.name,    // Name
             web_color.rgb.0, web_color.rgb.1, web_color.rgb.2, // Web colour block
             owo_block, // owo 256 color block
             owo_rgb.0, owo_rgb.1, owo_rgb.2, // owo rgb block
             format!("{owo_block:?}"),
             owo,   // #0-255
             web_color.rgb.0, web_color.rgb.1, web_color.rgb.2,    // From RGB
             owo_rgb.0, owo_rgb.1, owo_rgb.2     // to RGB
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
