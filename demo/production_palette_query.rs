/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/

/// Production-Ready Terminal Palette Query
///
/// This script demonstrates a production-ready implementation of OSC 4 palette
/// querying using the crossterm method, which has been proven to work reliably
/// across all major macOS terminals (Zed, WezTerm, Apple Terminal, iTerm2,
/// Alacritty, and Kitty).
///
/// Unlike the experimental version, this focuses on the reliable crossterm
/// approach and includes proper error handling, caching, and integration
/// patterns suitable for use in the thag_styling subcrate.
//# Purpose: Production-ready palette querying with crossterm
//# Categories: color, styling, terminal
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thag_styling::{ColorInitStrategy, ColorValue, Style, TermAttributes};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Calculate color difference (Manhattan distance in RGB space)
    pub fn distance_to(&self, other: &Rgb) -> u16 {
        ((self.r as i16 - other.r as i16).abs()
            + (self.g as i16 - other.g as i16).abs()
            + (self.b as i16 - other.b as i16).abs()) as u16
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Calculate perceived brightness (0.0-1.0)
    pub fn brightness(&self) -> f32 {
        // Using standard luminance formula
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    /// Check if this is a "dark" color
    pub fn is_dark(&self) -> bool {
        self.brightness() < 0.5
    }
}

/// Error types for palette querying
#[derive(Debug)]
pub enum PaletteError {
    Io(io::Error),
    Timeout,
    ParseError(String),
    ThreadError,
}

impl From<io::Error> for PaletteError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Cached palette query results
#[derive(Debug, Clone)]
pub struct PaletteCache {
    colors: HashMap<u8, Rgb>,
    timestamp: Instant,
    terminal_id: String,
}

impl PaletteCache {
    fn new(terminal_id: String) -> Self {
        Self {
            colors: HashMap::new(),
            timestamp: Instant::now(),
            terminal_id,
        }
    }

    fn is_valid(&self, max_age: Duration, current_terminal: &str) -> bool {
        self.timestamp.elapsed() < max_age && self.terminal_id == current_terminal
    }

    fn get_color(&self, index: u8) -> Option<Rgb> {
        self.colors.get(&index).copied()
    }

    fn set_color(&mut self, index: u8, color: Rgb) {
        self.colors.insert(index, color);
    }

    fn get_all_colors(&self) -> Vec<Option<Rgb>> {
        (0..16).map(|i| self.colors.get(&i).copied()).collect()
    }
}

/// Production-ready palette color query using crossterm threading
pub fn query_palette_color(index: u8, timeout: Duration) -> Result<Rgb, PaletteError> {
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle the terminal I/O
    let handle = thread::spawn(move || {
        let result = (|| -> Result<Rgb, PaletteError> {
            // Enable raw mode
            enable_raw_mode().map_err(PaletteError::Io)?;

            let mut stdout = io::stdout();
            let mut stdin = io::stdin();

            // Send query
            let query = format!("\x1b]4;{};?\x07", index);
            stdout.write_all(query.as_bytes())?;
            stdout.flush()?;

            // Try to read response
            let mut buffer = Vec::new();
            let mut temp_buffer = [0u8; 1];
            let start = Instant::now();

            while start.elapsed() < timeout {
                match stdin.read(&mut temp_buffer) {
                    Ok(1..) => {
                        buffer.push(temp_buffer[0]);

                        // Try to parse response
                        let response = String::from_utf8_lossy(&buffer);
                        if let Some(rgb) = try_parse_osc4_response(&response, index) {
                            return Ok(rgb);
                        }

                        // Prevent buffer overflow
                        if buffer.len() > 256 {
                            break;
                        }
                    }
                    Ok(0) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => return Err(PaletteError::Io(e)),
                }
            }

            Err(PaletteError::Timeout)
        })();

        // Always disable raw mode
        let _ = disable_raw_mode();

        let _ = tx.send(result);
    });

    // Wait for result or timeout
    match rx.recv_timeout(timeout + Duration::from_millis(50)) {
        Ok(result) => {
            let _ = handle.join();
            result
        }
        Err(_) => Err(PaletteError::ThreadError),
    }
}

/// Parse OSC 4 response from accumulated buffer
fn try_parse_osc4_response(response: &str, expected_index: u8) -> Option<Rgb> {
    let pattern = format!("\x1b]4;{};", expected_index);

    if let Some(start_pos) = response.find(&pattern) {
        let response_part = &response[start_pos..];

        // Handle rgb:RRRR/GGGG/BBBB format
        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos + 4..];

            let end_pos = rgb_data
                .find('\x07')
                .or_else(|| rgb_data.find('\x1b'))
                .unwrap_or(rgb_data.len().min(20));

            let rgb_str = &rgb_data[..end_pos];
            let parts: Vec<&str> = rgb_str.split('/').collect();

            if parts.len() == 3
                && parts[0].len() == 4
                && parts[1].len() == 4
                && parts[2].len() == 4
                && parts
                    .iter()
                    .all(|part| part.chars().all(|c| c.is_ascii_hexdigit()))
            {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parse_hex_component(parts[0]),
                    parse_hex_component(parts[1]),
                    parse_hex_component(parts[2]),
                ) {
                    return Some(Rgb::new(r, g, b));
                }
            }
        }

        // Handle #RRGGBB format
        if let Some(hash_pos) = response_part.find('#') {
            let hex_data = &response_part[hash_pos + 1..];
            if hex_data.len() >= 6 {
                let hex_str = &hex_data[..6];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex_str[0..2], 16),
                    u8::from_str_radix(&hex_str[2..4], 16),
                    u8::from_str_radix(&hex_str[4..6], 16),
                ) {
                    return Some(Rgb::new(r, g, b));
                }
            }
        }
    }

    None
}

/// Parse hex component (2 or 4 digits)
fn parse_hex_component(hex_str: &str) -> Result<u8, std::num::ParseIntError> {
    let clean_hex: String = hex_str
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect();

    match clean_hex.len() {
        4 => {
            // 16-bit value, take high byte
            let val = u16::from_str_radix(&clean_hex, 16)?;
            Ok((val >> 8) as u8)
        }
        2 => {
            // 8-bit value
            u8::from_str_radix(&clean_hex, 16)
        }
        _ => {
            // Try to parse whatever we have
            let val = u16::from_str_radix(&clean_hex, 16).unwrap_or(0);
            Ok((val.min(255)) as u8)
        }
    }
}

/// Get terminal identifier for caching
fn get_terminal_id() -> String {
    let mut id_parts = Vec::new();

    if let Ok(term) = std::env::var("TERM") {
        id_parts.push(format!("TERM={}", term));
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        id_parts.push(format!("PROGRAM={}", term_program));
    }
    if let Ok(term_version) = std::env::var("TERM_PROGRAM_VERSION") {
        id_parts.push(format!("VERSION={}", term_version));
    }

    if id_parts.is_empty() {
        "unknown".to_string()
    } else {
        id_parts.join("|")
    }
}

/// Production-ready palette detection with caching
pub struct PaletteDetector {
    cache: Option<PaletteCache>,
    timeout: Duration,
    cache_duration: Duration,
}

impl Default for PaletteDetector {
    fn default() -> Self {
        Self {
            cache: None,
            timeout: Duration::from_millis(100),
            cache_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl PaletteDetector {
    pub fn new(timeout: Duration, cache_duration: Duration) -> Self {
        Self {
            cache: None,
            timeout,
            cache_duration,
        }
    }

    /// Query a specific palette color with caching
    pub fn get_color(&mut self, index: u8) -> Option<Rgb> {
        if index >= 16 {
            return None;
        }

        let terminal_id = get_terminal_id();

        // Check cache first
        if let Some(cache) = &self.cache {
            if cache.is_valid(self.cache_duration, &terminal_id) {
                if let Some(color) = cache.get_color(index) {
                    return Some(color);
                }
            }
        }

        // Query from terminal
        match query_palette_color(index, self.timeout) {
            Ok(color) => {
                // Update cache
                if self.cache.is_none()
                    || !self
                        .cache
                        .as_ref()
                        .unwrap()
                        .is_valid(self.cache_duration, &terminal_id)
                {
                    self.cache = Some(PaletteCache::new(terminal_id));
                }

                if let Some(cache) = &mut self.cache {
                    cache.set_color(index, color);
                }

                Some(color)
            }
            Err(_) => None,
        }
    }

    /// Query all 16 palette colors
    pub fn get_all_colors(&mut self) -> Vec<Option<Rgb>> {
        let terminal_id = get_terminal_id();

        // Check if we have a valid complete cache
        if let Some(cache) = &self.cache {
            if cache.is_valid(self.cache_duration, &terminal_id) && cache.colors.len() == 16 {
                return cache.get_all_colors();
            }
        }

        // Query all colors
        let mut colors = Vec::with_capacity(16);
        let mut cache = PaletteCache::new(terminal_id);

        for i in 0..16 {
            match query_palette_color(i, self.timeout) {
                Ok(color) => {
                    colors.push(Some(color));
                    cache.set_color(i, color);
                }
                Err(_) => {
                    colors.push(None);
                }
            }

            // Small delay to avoid overwhelming terminal
            thread::sleep(Duration::from_millis(5));
        }

        self.cache = Some(cache);
        colors
    }

    /// Clear the cache (force re-query)
    pub fn clear_cache(&mut self) {
        self.cache = None;
    }

    /// Get cache statistics
    pub fn cache_info(&self) -> Option<(usize, Duration, String)> {
        self.cache.as_ref().map(|cache| {
            (
                cache.colors.len(),
                cache.timestamp.elapsed(),
                cache.terminal_id.clone(),
            )
        })
    }
}

/// Compare palette colors with current thag theme
pub fn compare_with_theme(colors: &[Option<Rgb>]) -> Vec<(u8, String, Option<u16>)> {
    let term_attrs = TermAttributes::get_or_init();
    let theme = &term_attrs.theme;
    let mut comparisons = Vec::new();

    // Define role mappings (approximation)
    let role_mappings = [
        (
            0,
            "Background",
            theme.bg_rgbs.first().copied(),
            "Terminal background",
        ),
        (
            1,
            "Emphasis",
            extract_rgb_from_style(&theme.palette.emphasis),
            "Emphasis",
        ),
        (
            2,
            "Success",
            extract_rgb_from_style(&theme.palette.success),
            "Success messages",
        ),
        (
            3,
            "Commentary",
            extract_rgb_from_style(&theme.palette.commentary),
            "Commentary text",
        ),
        (
            4,
            "Info",
            extract_rgb_from_style(&theme.palette.info),
            "Information",
        ),
        (
            5,
            "Heading 1",
            extract_rgb_from_style(&theme.palette.heading1),
            "Main Headings",
        ),
        (
            6,
            "Code",
            extract_rgb_from_style(&theme.palette.code),
            "Code samples",
        ),
        (
            7,
            "Normal",
            extract_rgb_from_style(&theme.palette.normal),
            "Normal text",
        ),
        (
            8,
            "Subtle",
            extract_rgb_from_style(&theme.palette.subtle),
            "Subtle text",
        ),
        (
            9,
            "Error",
            extract_rgb_from_style(&theme.palette.error),
            "Errors",
        ),
        (
            10,
            "Debug",
            extract_rgb_from_style(&theme.palette.debug),
            "Debug text",
        ),
        (
            11,
            "Warning",
            extract_rgb_from_style(&theme.palette.warning),
            "Warnings",
        ),
        (
            12,
            "Link",
            extract_rgb_from_style(&theme.palette.link),
            "Links",
        ),
        (
            13,
            "Heading 2",
            extract_rgb_from_style(&theme.palette.heading2),
            "Headings 2",
        ),
        (
            14,
            "Hint",
            extract_rgb_from_style(&theme.palette.hint),
            "Hints",
        ),
        (
            15,
            "Quote",
            extract_rgb_from_style(&theme.palette.quote),
            "Quoted text",
        ),
    ];

    for (ansi_index, role_name, theme_rgb_opt, _description) in role_mappings {
        if let (Some(Some(queried_rgb)), Some(theme_rgb)) = (colors.get(ansi_index), theme_rgb_opt)
        {
            eprintln!("queried_rgb={queried_rgb:?}, theme_rgb={theme_rgb:?}");
            let theme_rgb_struct = Rgb::new(theme_rgb.0, theme_rgb.1, theme_rgb.2);
            let distance = queried_rgb.distance_to(&theme_rgb_struct);

            comparisons.push((ansi_index as u8, role_name.to_string(), Some(distance)));
        } else {
            comparisons.push((ansi_index as u8, role_name.to_string(), None));
        }
    }

    comparisons
}

/// Extract RGB from a thag Style
fn extract_rgb_from_style(style: &Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

/// Display palette colors in a formatted table
fn display_palette(colors: &[Option<Rgb>]) {
    println!("🎨 Terminal Palette Colors");
    println!("==========================");

    let color_names = [
        "Black",
        "Red",
        "Green",
        "Yellow",
        "Blue",
        "Magenta",
        "Cyan",
        "White",
        "Br Black",
        "Br Red",
        "Br Green",
        "Br Yellow",
        "Br Blue",
        "Br Magenta",
        "Br Cyan",
        "Br White",
    ];

    for (i, (name, color_opt)) in color_names.iter().zip(colors.iter()).enumerate() {
        print!("{:2}: {:>10} ", i, name);

        if let Some(color) = color_opt {
            print!(
                "RGB({:3},{:3},{:3}) {} \x1b[38;5;{}m█████\x1b[0m \x1b[48;5;{}m     \x1b[0m",
                color.r,
                color.g,
                color.b,
                color.to_hex(),
                i,
                i
            );

            // Add brightness indicator
            if color.is_dark() {
                print!(" 🌙");
            } else {
                print!(" ☀️");
            }
        } else {
            print!("Not available");
        }

        println!();
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Production-Ready Terminal Palette Query");
    println!("===========================================");
    println!("This script uses the proven crossterm method for reliable palette detection.");
    println!();

    let term_attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);
    let theme = &term_attrs.theme;
    println!("Current theme: {}", theme.name);

    // Show terminal info
    println!("🖥️  Terminal Environment:");
    if let Ok(term) = std::env::var("TERM") {
        println!("   TERM: {}", term);
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        println!("   PROGRAM: {}", term_program);
    }
    println!();

    // Create detector with production settings
    let mut detector = PaletteDetector::new(
        Duration::from_millis(150), // Reasonable timeout
        Duration::from_secs(300),   // 5-minute cache
    );

    println!("🔍 Querying palette colors...");
    let start_time = Instant::now();
    let colors = detector.get_all_colors();
    let query_time = start_time.elapsed();

    let successful = colors.iter().filter(|c| c.is_some()).count();
    println!(
        "✅ Query completed in {:?}: {}/16 colors detected",
        query_time, successful
    );
    println!();

    if successful > 0 {
        display_palette(&colors);

        // Theme comparison
        let comparisons = compare_with_theme(&colors);
        println!("🎯 Theme Compatibility Analysis:");
        println!("================================");

        // let term_attrs = TermAttributes::get_or_init();
        println!("Current theme: {}", term_attrs.theme.name);
        println!();

        for (index, role, distance_opt) in comparisons {
            if let Some(distance) = distance_opt {
                let compatibility = match distance {
                    0..=20 => "✅ Excellent match",
                    21..=60 => "🟡 Good compatibility",
                    61..=150 => "🟠 Moderate difference",
                    _ => "🔴 Poor match",
                };
                println!(
                    "Color {:2} ({}): {} (distance: {})",
                    index, role, compatibility, distance
                );
            } else {
                println!("Color {:2} ({}): ❌ Not available", index, role);
            }
        }
        println!();

        // Cache info
        if let Some((cached_colors, age, terminal_id)) = detector.cache_info() {
            println!("💾 Cache Information:");
            println!("   Cached colors: {}/16", cached_colors);
            println!("   Cache age: {:?}", age);
            println!("   Terminal ID: {}", terminal_id);
            println!();
        }

        // Recommendations
        println!("💡 Integration Recommendations:");
        println!("   • Use palette detection for theme auto-adjustment");
        println!("   • Cache results to avoid repeated queries");
        println!("   • Implement graceful fallbacks for unsupported terminals");
        println!("   • Consider user preferences and manual overrides");
    } else {
        println!("❌ No palette colors could be detected.");
        println!("This could indicate:");
        println!("  • Terminal doesn't support OSC 4 queries");
        println!("  • Running in a restricted environment");
        println!("  • Terminal multiplexer blocking sequences");
        println!();
        println!("💡 Consider fallback detection methods:");
        println!("  • Environment variable analysis");
        println!("  • Background color detection (termbg)");
        println!("  • User configuration options");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_operations() {
        let color1 = Rgb::new(255, 128, 64);
        let color2 = Rgb::new(255, 128, 64);

        assert_eq!(color1.distance_to(&color2), 0);
        assert_eq!(color1.to_hex(), "#ff8040");
        assert!(!color1.is_dark()); // Should be bright

        let dark_color = Rgb::new(32, 32, 32);
        assert!(dark_color.is_dark());
    }

    #[test]
    fn test_parse_osc4_response() {
        // Test RGB format
        let response1 = "\x1b]4;1;rgb:ff00/8000/4000\x07";
        let result1 = try_parse_osc4_response(response1, 1);
        assert_eq!(result1, Some(Rgb::new(255, 128, 64)));

        // Test hex format
        let response2 = "\x1b]4;0;#ff8040\x07";
        let result2 = try_parse_osc4_response(response2, 0);
        assert_eq!(result2, Some(Rgb::new(255, 128, 64)));
    }

    #[test]
    fn test_cache_functionality() {
        let mut cache = PaletteCache::new("test_terminal".to_string());
        let color = Rgb::new(255, 0, 0);

        cache.set_color(1, color);
        assert_eq!(cache.get_color(1), Some(color));
        assert_eq!(cache.get_color(2), None);

        assert!(cache.is_valid(Duration::from_secs(1), "test_terminal"));
        assert!(!cache.is_valid(Duration::from_secs(1), "different_terminal"));
    }

    #[test]
    fn test_parse_hex_component() {
        assert_eq!(parse_hex_component("ff").unwrap(), 255);
        assert_eq!(parse_hex_component("ff00").unwrap(), 255);
        assert_eq!(parse_hex_component("8000").unwrap(), 128);
    }
}
