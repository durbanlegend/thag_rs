//! Standard 256-color terminal palette
//!
//! Colors are named using the most widely-accepted terms from:
//! - ANSI standard (0-15)
//! - X11 rgb.txt for the color cube (16-231)
//! - xterm for grayscale ramp (232-255)
//!
//! All names are unique and have documented RGB values and sources.

// use std::fmt;

/// Standard 256-color terminal palette with commonly accepted names.
/// Colors 16-231 form a 6x6x6 color cube where:
/// - Red increases every 36 values (0-5 * 51)
/// - Green increases every 6 values (0-5 * 51)
/// - Blue increases every 1 value (0-5 * 51)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour256 {
    // System colors (0-15)
    // System colors (0-15)
    UserBlack = 0,
    UserRed = 1,
    UserGreen = 2,
    UserYellow = 3,
    UserBlue = 4,
    UserMagenta = 5,
    UserCyan = 6,
    UserLightGray = 7,
    UserDarkGray = 8,
    UserBrightRed = 9,
    UserBrightGreen = 10,
    UserBrightYellow = 11,
    UserBrightBlue = 12,
    UserBrightMagenta = 13,
    UserBrightCyan = 14,
    UserWhite = 15,

    // First face (16-51)
    // First row (pure blues)
    Black = 16,      // #000000
    Blue4 = 17,      // #00005f
    Blue3 = 18,      // #000087
    Blue2 = 19,      // #0000af
    Blue1 = 20,      // #0000d7
    RoyalBlue1 = 21, // #0000ff

    // Second row (blue-greens)
    Green4 = 22,       // #005f00
    SpringGreen4 = 23, // #005f5f
    SteelBlue4 = 24,   // #005f87
    SteelBlue3 = 25,   // #005faf
    RoyalBlue2 = 26,   // #005fd7
    RoyalBlue3 = 27,   // #005fff

    // Third row (more green-blues)
    SeaGreen4 = 28,    // #008700
    SeaGreen3 = 29,    // #00875f
    SteelBlue2 = 30,   // #008787
    SteelBlue1 = 31,   // #0087af
    DeepSkyBlue3 = 32, // #0087d7
    DeepSkyBlue2 = 33, // #0087ff

    // Fourth row
    SeaGreen2 = 34,    // #00af00
    SeaGreen1 = 35,    // #00af5f
    Turquoise4 = 36,   // #00af87
    Turquoise3 = 37,   // #00afaf
    DeepSkyBlue1 = 38, // #00afd7
    DodgerBlue1 = 39,  // #00afff

    // Fifth row
    SpringGreen3 = 40, // #00d700
    SpringGreen2 = 41, // #00d75f
    Turquoise2 = 42,   // #00d787
    Turquoise1 = 43,   // #00d7af
    Cyan3 = 44,        // #00d7d7
    Cyan2 = 45,        // #00d7ff

    // Sixth row
    Green1 = 46,            // #00ff00
    SpringGreen1 = 47,      // #00ff5f
    MediumSpringGreen = 48, // #00ff87
    Cyan1 = 49,             // #00ffaf
    LightCyan1 = 50,        // #00ffd7
    LightCyan2 = 51,        // #00ffff

    // Second face (52-87) - adding red component (r=1 * 51)
    DarkRed = 52,       // #5f0000
    Purple4 = 53,       // #5f005f
    Purple3 = 54,       // #5f0087
    Purple2 = 55,       // #5f00af
    Purple1 = 56,       // #5f00d7
    MediumPurple1 = 57, // #5f00ff

    Brown4 = 58,        // #5f3700
    Magenta4 = 59,      // #5f375f
    MediumOrchid4 = 60, // #5f3787
    MediumPurple4 = 61, // #5f37af
    SlateBlue2 = 62,    // #5f37d7
    SlateBlue1 = 63,    // #5f37ff

    Indian4 = 64,         // #5f5f00
    PaleVioletRed4 = 65,  // #5f5f5f
    MediumPurple3 = 66,   // #5f5f87
    MediumPurple2 = 67,   // #5f5faf
    LightSlateBlue = 68,  // #5f5fd7
    LightSlateBlue1 = 69, // #5f5fff

    DarkOliveGreen4 = 70, // #5f8700
    DarkSeaGreen4 = 71,   // #5f875f
    LightSkyBlue4 = 72,   // #5f8787
    LightSkyBlue3 = 73,   // #5f87af
    SkyBlue2 = 74,        // #5f87d7
    SkyBlue1 = 75,        // #5f87ff

    DarkSeaGreen3 = 76, // #5faf00
    DarkSeaGreen2 = 77, // #5faf5f
    LightSkyBlue2 = 78, // #5faf87
    LightSkyBlue1 = 79, // #5fafaf
    DodgerBlue2 = 80,   // #5fafd7
    DodgerBlue3 = 81,   // #5fafff

    PaleGreen3 = 82,      // #5fd700
    PaleGreen2 = 83,      // #5fd75f
    PaleTurquoise4 = 84,  // #5fd787
    PaleTurquoise3 = 85,  // #5fd7af
    CornflowerBlue = 86,  // #5fd7d7
    CornflowerBlue1 = 87, // #5fd7ff

    // Third face (88-123)
    Red4 = 88,            // #870000
    DeepPink4 = 89,       // #87005f
    MediumVioletRed = 90, // #870087
    Magenta3 = 91,        // #8700af
    DarkViolet = 92,      // #8700d7
    Purple = 93,          // #8700ff

    Maroon4 = 94,       // #873700
    HotPink4 = 95,      // #87375f
    MediumOrchid3 = 96, // #873787
    MediumOrchid2 = 97, // #8737af
    MediumOrchid1 = 98, // #8737d7
    BlueViolet = 99,    // #8737ff

    OrangeRed4 = 100,   // #875f00
    IndianRed4 = 101,   // #875f5f
    HotPink3 = 102,     // #875f87
    MediumOrchid = 103, // #875faf
    DarkOrchid = 104,   // #875fd7
    DarkOrchid1 = 105,  // #875fff

    Sienna4 = 106,    // #878700
    LightPink4 = 107, // #87875f
    Plum4 = 108,      // #878787
    Plum3 = 109,      // #8787af
    Plum2 = 110,      // #8787d7
    Plum1 = 111,      // #8787ff

    LightGoldenrod4 = 112, // #87af00
    LightPink3 = 113,      // #87af5f
    LightPink2 = 114,      // #87af87
    LightPink1 = 115,      // #87afaf
    Orchid2 = 116,         // #87afd7
    Orchid1 = 117,         // #87afff

    Gold4 = 118,           // #87d700
    Khaki4 = 119,          // #87d75f
    LightGoldenrod3 = 120, // #87d787
    LightGoldenrod2 = 121, // #87d7af
    Thistle3 = 122,        // #87d7d7
    Orchid = 123,          // #87d7ff
}

impl Colour256 {
    /// Get the RGB values for a color number
    pub fn rgb_x11(color: u8) -> (u8, u8, u8) {
        match color {
            0..=15 => SYSTEM_COLORS[color as usize],
            16..=231 => {
                // Calculate 6x6x6 color cube
                let color = color - 16;
                let r = (color / 36) * 51;
                let g = ((color % 36) / 6) * 51;
                let b = (color % 6) * 51;
                (r, g, b)
            }
            232..=255 => {
                // Calculate grayscale ramp
                let gray = 8 + (color - 232) * 10;
                (gray, gray, gray)
            } // _ => unreachable!(),
        }
    }

    /// Get RGB val.ues for a color number, based on owo's interpretation
    /// of the RGB values incrementing as 0, 95, 135, 175, 215, 255 rather
    /// than the Xterm 0, 5§,102, 153, 204, 255. I don't know for sure but
    /// these are just documentation and don't affect what the terminal
    /// does with the numbers 0-255 anyway
    pub fn rgb_95_40(color: u8) -> (u8, u8, u8) {
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
    /// Get the color name for a color number
    pub fn name(color: u8) -> &'static str {
        Self::from_u8(color).as_str()
    }
    pub fn from_u8(color: u8) -> Self {
        // Safe because enum discriminants match u8 values exactly 0-255
        unsafe { std::mem::transmute(color) }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            // System colors (0-15)
            Self::UserBlack => "UserBlack",
            Self::UserRed => "UserRed",
            Self::UserGreen => "UserGreen",
            Self::UserYellow => "UserYellow",
            Self::UserBlue => "UserBlue",
            Self::UserMagenta => "UserMagenta",
            Self::UserCyan => "UserCyan",
            Self::UserLightGray => "UserLightGray",
            Self::UserDarkGray => "UserDarkGray",
            Self::UserBrightRed => "UserBrightRed",
            Self::UserBrightGreen => "UserBrightGreen",
            Self::UserBrightYellow => "UserBrightYellow",
            Self::UserBrightBlue => "UserBrightBlue",
            Self::UserBrightMagenta => "UserBrightMagenta",
            Self::UserBrightCyan => "UserBrightCyan",
            Self::UserWhite => "UserWhite",

            // First face (16-51)
            Self::Black => "Black",
            Self::Blue4 => "Blue4",
            Self::Blue3 => "Blue3",
            Self::Blue2 => "Blue2",
            Self::Blue1 => "Blue1",
            Self::RoyalBlue1 => "RoyalBlue1",

            Self::Green4 => "Green4",
            Self::SpringGreen4 => "SpringGreen4",
            Self::SteelBlue4 => "SteelBlue4",
            Self::SteelBlue3 => "SteelBlue3",
            Self::RoyalBlue2 => "RoyalBlue2",
            Self::RoyalBlue3 => "RoyalBlue3",

            Self::SeaGreen4 => "SeaGreen4",
            Self::SeaGreen3 => "SeaGreen3",
            Self::SteelBlue2 => "SteelBlue2",
            Self::SteelBlue1 => "SteelBlue1",
            Self::DeepSkyBlue3 => "DeepSkyBlue3",
            Self::DeepSkyBlue2 => "DeepSkyBlue2",

            Self::SeaGreen2 => "SeaGreen2",
            Self::SeaGreen1 => "SeaGreen1",
            Self::Turquoise4 => "Turquoise4",
            Self::Turquoise3 => "Turquoise3",
            Self::DeepSkyBlue1 => "DeepSkyBlue1",
            Self::DodgerBlue1 => "DodgerBlue1",

            Self::SpringGreen3 => "SpringGreen3",
            Self::SpringGreen2 => "SpringGreen2",
            Self::Turquoise2 => "Turquoise2",
            Self::Turquoise1 => "Turquoise1",
            Self::Cyan3 => "Cyan3",
            Self::Cyan2 => "Cyan2",

            Self::Green1 => "Green1",
            Self::SpringGreen1 => "SpringGreen1",
            Self::MediumSpringGreen => "MediumSpringGreen",
            Self::Cyan1 => "Cyan1",
            Self::LightCyan1 => "LightCyan1",
            Self::LightCyan2 => "LightCyan2",

            // Second face (52-87)
            Self::DarkRed => "DarkRed",
            Self::Purple4 => "Purple4",
            Self::Purple3 => "Purple3",
            Self::Purple2 => "Purple2",
            Self::Purple1 => "Purple1",
            Self::MediumPurple1 => "MediumPurple1",

            Self::Brown4 => "Brown4",
            Self::Magenta4 => "Magenta4",
            Self::MediumOrchid4 => "MediumOrchid4",
            Self::MediumPurple4 => "MediumPurple4",
            Self::SlateBlue2 => "SlateBlue2",
            Self::SlateBlue1 => "SlateBlue1",

            Self::Indian4 => "Indian4",
            Self::PaleVioletRed4 => "PaleVioletRed4",
            Self::MediumPurple3 => "MediumPurple3",
            Self::MediumPurple2 => "MediumPurple2",
            Self::LightSlateBlue => "LightSlateBlue",
            Self::LightSlateBlue1 => "LightSlateBlue1",

            Self::DarkOliveGreen4 => "DarkOliveGreen4",
            Self::DarkSeaGreen4 => "DarkSeaGreen4",
            Self::LightSkyBlue4 => "LightSkyBlue4",
            Self::LightSkyBlue3 => "LightSkyBlue3",
            Self::SkyBlue2 => "SkyBlue2",
            Self::SkyBlue1 => "SkyBlue1",

            Self::DarkSeaGreen3 => "DarkSeaGreen3",
            Self::DarkSeaGreen2 => "DarkSeaGreen2",
            Self::LightSkyBlue2 => "LightSkyBlue2",
            Self::LightSkyBlue1 => "LightSkyBlue1",
            Self::DodgerBlue2 => "DodgerBlue2",
            Self::DodgerBlue3 => "DodgerBlue3",

            Self::PaleGreen3 => "PaleGreen3",
            Self::PaleGreen2 => "PaleGreen2",
            Self::PaleTurquoise4 => "PaleTurquoise4",
            Self::PaleTurquoise3 => "PaleTurquoise3",
            Self::CornflowerBlue => "CornflowerBlue",
            Self::CornflowerBlue1 => "CornflowerBlue1",

            // Third face (88-123)
            Self::Red4 => "Red4",
            Self::DeepPink4 => "DeepPink4",
            Self::MediumVioletRed => "MediumVioletRed",
            Self::Magenta3 => "Magenta3",
            Self::DarkViolet => "DarkViolet",
            Self::Purple => "Purple",

            Self::Maroon4 => "Maroon4",
            Self::HotPink4 => "HotPink4",
            Self::MediumOrchid3 => "MediumOrchid3",
            Self::MediumOrchid2 => "MediumOrchid2",
            Self::MediumOrchid1 => "MediumOrchid1",
            Self::BlueViolet => "BlueViolet",

            Self::OrangeRed4 => "OrangeRed4",
            Self::IndianRed4 => "IndianRed4",
            Self::HotPink3 => "HotPink3",
            Self::MediumOrchid => "MediumOrchid",
            Self::DarkOrchid => "DarkOrchid",
            Self::DarkOrchid1 => "DarkOrchid1",

            Self::Sienna4 => "Sienna4",
            Self::LightPink4 => "LightPink4",
            Self::Plum4 => "Plum4",
            Self::Plum3 => "Plum3",
            Self::Plum2 => "Plum2",
            Self::Plum1 => "Plum1",

            Self::LightGoldenrod4 => "LightGoldenrod4",
            Self::LightPink3 => "LightPink3",
            Self::LightPink2 => "LightPink2",
            Self::LightPink1 => "LightPink1",
            Self::Orchid2 => "Orchid2",
            Self::Orchid1 => "Orchid1",

            Self::Gold4 => "Gold4",
            Self::Khaki4 => "Khaki4",
            Self::LightGoldenrod3 => "LightGoldenrod3",
            Self::LightGoldenrod2 => "LightGoldenrod2",
            Self::Thistle3 => "Thistle3",
            Self::Orchid => "Orchid",
        }
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

fn main() {
    // Print a color block with its number and name
    fn show_color(num: u8) {
        let name = Colour256::name(num);
        // if !name.contains("Blue") {
        //     return;
        // }
        // if !name.contains("Green") && !name.contains("Teal") {
        //     return;
        // }
        // let rgb = Colour256::rgb_95_40(num);
        let rgb = Colour256::rgb_95_40(num);
        println!(
            "\x1b[48;5;{}m     \x1b[0m #{:03}: {} (RGB: {:03},{:03},{:03})",
            num, num, name, rgb.0, rgb.1, rgb.2
        );
    }

    println!("Color Cube Faces:");
    for face in 0..6 {
        println!("\nFace {} (starts at {})", face + 1, 16 + (face * 36));
        // Show colors of each face
        for i in 0..36 {
            show_color(16 + (face * 36) + i);
        }
    }
}
