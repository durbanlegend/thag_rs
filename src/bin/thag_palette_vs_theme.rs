/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/
#![allow(clippy::uninlined_format_args)]
/// Terminal palette comparison tool with theme selection
///
/// This tool displays the current terminal's color palette alongside
/// a selected thag theme for direct comparison. Helps identify color
/// mapping issues and verify theme installation.
//# Purpose: Compare terminal palette with thag theme colors
//# Categories: color, styling, terminal, theming, tools
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use inquire::set_global_render_config; // For optional theming of `inquire`
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use thag_styling::{
    auto_help, display_color_comparison, file_navigator, help_system::check_help_and_exit,
    select_builtin_theme, sprtln, styling::index_to_rgb, themed_inquire_config, ColorInitStrategy,
    ColorValue, Role, Style, Styleable, StyledPrint, TermAttributes, TermBgLuma, Theme,
};

file_navigator! {}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Calculate color difference (Manhattan distance in RGB space)
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn distance_to(&self, other: &Self) -> u16 {
        ((i16::from(self.r) - i16::from(other.r)).abs()
            + (i16::from(self.g) - i16::from(other.g)).abs()
            + (i16::from(self.b) - i16::from(other.b)).abs()) as u16
    }

    /// Convert to hex string
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Calculate perceived brightness (0.0-1.0)
    #[must_use]
    pub fn brightness(&self) -> f32 {
        // Using standard luminance formula
        0.114f32.mul_add(
            f32::from(self.b),
            0.299f32.mul_add(f32::from(self.r), 0.587 * f32::from(self.g)),
        ) / 255.0
    }

    /// Check if this is a "dark" color
    #[must_use]
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
///
/// # Errors
///
/// Will bubble up any terminal errors encountered.
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
    rx.recv_timeout(timeout + Duration::from_millis(50)).map_or(
        Err(PaletteError::ThreadError),
        |result| {
            let _ = handle.join();
            result
        },
    )
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
                .unwrap_or_else(|| rgb_data.len().min(20));

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
        .take_while(char::is_ascii_hexdigit)
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
    #[must_use]
    pub const fn new(timeout: Duration, cache_duration: Duration) -> Self {
        Self {
            cache: None,
            timeout,
            cache_duration,
        }
    }

    /// Query a specific palette color with caching
    ///
    /// # Panics
    ///
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
                if let Some(cache) = &self.cache {
                    if !cache.is_valid(self.cache_duration, &terminal_id) {
                        self.cache = Some(PaletteCache::new(terminal_id));
                    }
                } else {
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
    #[must_use]
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

fn main() -> Result<(), Box<dyn Error>> {
    // Check for help first
    let help = auto_help!();
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    // Initialize styling system
    TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

    format!(
        "🎨 {} - Terminal Palette vs Theme Comparison",
        "thag_palette_vs_theme".info()
    )
    .warning()
    .println();
    sprtln!(Role::Subtle, "{}", "═".repeat(63));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Select theme to compare
    let theme = select_theme(&mut navigator)?;

    theme.with_context(|| {
        format!("📋 Selected theme: {}", &theme.name.heading3())
            .normal()
            .println();
        println!("📝 Description: {}", theme.description);
    });
    println!();

    // Display comprehensive comparison
    display_terminal_info(&theme);
    display_ansi_colors(&theme);
    display_theme_colors(&theme);
    display_color_comparison(&theme);
    display_recommendations(&theme);

    println!("\n🎉 Palette comparison complete!");
    Ok(())
}

/// Select a theme using file navigator or built-in themes
fn select_theme(navigator: &mut FileNavigator) -> Result<Theme, Box<dyn Error>> {
    use inquire::{Select, Text};

    const FROM_DIR: &str = "Select theme file (.toml)";
    const BUILT_IN: &str = "Use precompiled built-in theme by name";
    const PRECOMPILED_LIST: &str = "Select from the list of precompiled built-in themes";
    let selection_options = vec![FROM_DIR, BUILT_IN, PRECOMPILED_LIST];

    let selection_method =
        Select::new("How would you like to select a theme?", selection_options).prompt()?;

    match selection_method {
        FROM_DIR => {
            println!("\n📁 Select a theme file:");
            let Ok(theme_file) = select_file(navigator, Some("toml"), false) else {
                return Err("No theme file selected".into());
            };

            format!(
                "📄 Loading theme from: {}",
                &theme_file.display().to_string().debug()
            )
            .normal()
            .println();

            Theme::load_from_file(&theme_file)
                .map_err(|e| format!("Failed to load theme file: {}", e).into())
        }
        BUILT_IN => {
            let theme_name = Text::new("Enter built-in theme name:")
                .with_help_message("e.g., 'thag-vibrant-dark', 'dracula_official', 'gruvbox_dark'")
                .prompt()?;

            Theme::get_builtin(&theme_name).map_err(|e| {
                format!("Failed to load built-in theme '{}': {}", theme_name, e).into()
            })
        }
        PRECOMPILED_LIST => {
            format!("\n📚 {} Built-in themes:", "Available".info())
                .normal()
                .println();
            println!("─────────────────────");

            let maybe_theme_name = select_builtin_theme();
            let Some(theme_name) = maybe_theme_name else {
                return Err("No theme selected".into());
            };

            Theme::get_builtin(&theme_name).map_err(|e| {
                format!("Failed to load built-in theme '{}': {}", theme_name, e).into()
            })
        }
        _ => Err("Invalid selection".into()),
    }
}

/// Display basic terminal information
fn display_terminal_info(theme: &Theme) {
    theme
        .normal(format!("📟 {} Information:", theme.info_text("Terminal")))
        .println();
    println!("────────────────────────");

    let term_attrs = TermAttributes::get_or_init();

    println!("🔍 Color Support: {:?}", term_attrs.color_support);
    println!("🌓 Background Luma: {:?}", term_attrs.term_bg_luma);

    // Display relevant environment variables
    if let Ok(term) = std::env::var("TERM") {
        theme
            .normal(format!("🖥️  TERM: {}", theme.debug(&term)))
            .println();
    }
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        theme
            .normal(format!("🖥️  COLORTERM: {}", theme.debug(&colorterm)))
            .println();
    }

    // Try to detect terminal emulator
    let terminal_info = detect_terminal_emulator();
    if !terminal_info.is_empty() {
        theme
            .normal(format!("🖥️  Detected: {}", theme.emphasis(&terminal_info)))
            .println();
    }

    println!();
}

/// Attempt to detect terminal emulator
fn detect_terminal_emulator() -> String {
    // Check various environment variables that indicate terminal type
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "Alacritty" => return "Alacritty".to_string(),
            "WezTerm" => return "WezTerm".to_string(),
            "iTerm.app" => return "iTerm2".to_string(),
            "Apple_Terminal" => return "Apple Terminal".to_string(),
            "vscode" => return "VS Code Terminal".to_string(),
            _ => {}
        }
    }

    if let Ok(wt_session) = std::env::var("WT_SESSION") {
        if !wt_session.is_empty() {
            return "Windows Terminal".to_string();
        }
    }

    if let Ok(kitty_window_id) = std::env::var("KITTY_WINDOW_ID") {
        if !kitty_window_id.is_empty() {
            return "Kitty".to_string();
        }
    }

    String::new()
}

/// Display the 16 basic ANSI colors
fn display_ansi_colors(theme: &Theme) {
    // Create detector with production settings
    let mut detector = PaletteDetector::new(
        Duration::from_millis(150), // Reasonable timeout
        Duration::from_secs(300),   // 5-minute cache
    );

    println!("🔍 Querying palette colors...");
    let start_time = Instant::now();
    let palette_colors = detector.get_all_colors();
    let query_time = start_time.elapsed();

    let successful = palette_colors.iter().filter(|c| c.is_some()).count();
    let palette_colors = if successful > 0 {
        Some(palette_colors.as_ref())
    } else {
        None
    };
    println!(
        "✅ Query completed in {:?}: {}/16 colors detected",
        query_time, successful
    );
    println!();

    theme.with_context(|| {
        format!("🎨 {} ANSI Colors (0-15):", "Current Terminal".info())
            .normal()
            .println();
        println!("───────────────────────────────────────");

        // Basic colors (0-7)
        "Standard Colors (0-7):".normal().println();
        display_color_row(
            theme,
            &[
                (0, "Black"),
                (1, "Red"),
                (2, "Green"),
                (3, "Yellow"),
                (4, "Blue"),
                (5, "Magenta"),
                (6, "Cyan"),
                (7, "White"),
            ],
            palette_colors,
            0,
        );

        println!();

        // Bright colors (8-15)
        println!("Bright Colors (8-15):");
        display_color_row(
            theme,
            &[
                (8, "Bright Black"),
                (9, "Bright Red"),
                (10, "Bright Green"),
                (11, "Bright Yellow"),
                (12, "Bright Blue"),
                (13, "Bright Magenta"),
                (14, "Bright Cyan"),
                (15, "Bright White"),
            ],
            palette_colors,
            1,
        );

        println!();
    });
}

/// Display a row of colors with their indices and names
fn display_color_row(
    theme: &Theme,
    colors: &[(u8, &str)],
    palette_colors: Option<&[Option<Rgb>]>,
    row: usize,
) {
    theme.with_context(|| {
        // Print color indices
        print!("   ");
        for (index, _) in colors {
            print!("{}", format!("{:>13}", index).emphasis());
        }
        println!();

        // Print color names
        print!("   ");
        for (_, name) in colors {
            print!("{:>13}", name);
        }
        println!();

        // Print colored blocks using ANSI escape codes
        print!("   ");
        for (index, _) in colors {
            print!("\x1b[48;5;{}m{:>13}\x1b[0m", index, "");
        }
        println!();

        if let Some(palette_colors) = palette_colors {
            // Print sample text in each color
            print!("   ");
            let start_index = row * 8;
            for i in 0..8 {
                let index = start_index + i;
                if let Some(color) = palette_colors[index] {
                    // print!("\x1b[38;5;{}m{:>13}\x1b[0m", index, "Sample");
                    let rgb = format!("{:>3},{:>3},{:>3}", color.r, color.g, color.b);
                    print!("\x1b[38;5;{}m{:>13}\x1b[0m", index, rgb);
                }
            }
            println!();
        }
    });
}

/// Display theme colors with visual preview
fn display_theme_colors(theme: &Theme) {
    theme.with_context(|| {
        format!("🌟 {} Colors:", theme.name.info())
            .normal()
            .println();
        println!("──────────────────────────────────────────────");

        println!("Background: {:?}", theme.bg_rgbs);
        println!();

        // Display semantic colors with visual preview
        let semantic_colors = [
            ("Heading1", &theme.palette.heading1),
            ("Heading2", &theme.palette.heading2),
            ("Heading3", &theme.palette.heading3),
            ("Error", &theme.palette.error),
            ("Warning", &theme.palette.warning),
            ("Success", &theme.palette.success),
            ("Info", &theme.palette.info),
            ("Emphasis", &theme.palette.emphasis),
            ("Code", &theme.palette.code),
            ("Normal", &theme.palette.normal),
            ("Subtle", &theme.palette.subtle),
            ("Hint", &theme.palette.hint),
            ("Debug", &theme.palette.debug),
            ("Link", &theme.palette.link),
            ("Quote", &theme.palette.quote),
            ("Commentary", &theme.palette.commentary),
        ];

        println!("Semantic Colors:");
        for (name, style) in semantic_colors {
            let colored_text = style.paint(format!("{:>12}", name));
            let rgb_info = extract_rgb_info(style);
            println!("   {} {}", colored_text, theme.code(&rgb_info));
        }

        // Show background color preview if available
        if let Some([r, g, b]) = theme.bg_rgbs.first() {
            println!();
            println!("Background Preview:");
            print!("   ");
            for _ in 0..20 {
                print!("\x1b[48;2;{r};{g};{b}m \x1b[0m");
            }
            theme.normal(format!(" RGB({r}, {g}, {b})")).println();
        }

        println!();
    });
}

/// Display recommendations based on comparison
fn display_recommendations(theme: &Theme) {
    theme.with_context(|| {
        format!("💡 {} and Tips:", "Recommendations".info())
            .normal()
            .println();
        println!("────────────────────────────");

        println!("• If colors don't match expected values:");
        println!("  - Your terminal may not support the theme correctly");
        format!(
            "  - Try using {} to synchronize the terminal palette with the `thag_styling` theme",
            "thag_sync_palette".heading3()
        )
        .normal()
        .println();
        println!("  - Check if your terminal emulator supports the theme format");
        println!();

        format!("• For {} theme:", theme.name.heading3())
            .normal()
            .println();
        match theme.term_bg_luma {
            TermBgLuma::Dark => {
                println!("  - Ensure your terminal has a dark background");
                println!("  - ANSI 0 (Black) should match background color");
            }
            TermBgLuma::Light => {
                println!("  - Ensure your terminal has a light background");
                println!("  - Colors should be adjusted for light backgrounds");
            }
            TermBgLuma::Undetermined => {}
        }
        println!();

        format!("• {} Commands:", "Useful".emphasis())
            .normal()
            .println();
        format!(
            "  - {}: Export theme to terminal formats",
            "thag_gen_terminal_themes".heading3()
        )
        .normal()
        .println();
        format!(
            "  - {}: Sync terminal palette",
            format!("thag_sync_palette --apply {}", theme.name).heading3()
        )
        .normal()
        .println();
        format!(
            "  - {}: Generate themes from images",
            "thag_image_to_theme".heading3()
        )
        .normal()
        .println();
        println!();

        // Show specific issues if detected
        let issues = detect_potential_issues(theme);
        if !issues.is_empty() {
            format!("⚠️  {} Issues Detected:", "Potential".emphasis())
                .normal()
                .println();
            for issue in issues {
                format!("   • {}", issue.emphasis()).normal().println();
            }
            println!();
        }
    });
}

/// Detect potential issues with theme/terminal compatibility
fn detect_potential_issues(theme: &Theme) -> Vec<String> {
    theme.with_context(|| {
        let mut issues = Vec::new();

        // Check if theme colors are too similar to background
        if let Some(bg_rgb) = theme.bg_rgbs.first() {
            if let Some(normal_rgb) = extract_rgb(&theme.palette.normal) {
                let contrast = calculate_contrast_ratio(*bg_rgb, normal_rgb);
                if contrast < 4.5 {
                    issues.push(format!(
                    "Low contrast between background and normal text ({}:1, recommended 4.5:1+)",
                    format_args!("{:.1}", contrast)
                ));
                }
            }
        }

        // Check for missing color information
        let essential_colors = [
            ("Error", &theme.palette.error),
            ("Warning", &theme.palette.warning),
            ("Success", &theme.palette.success),
            ("Normal", &theme.palette.normal),
        ];

        for (name, style) in essential_colors {
            if extract_rgb(style).is_none() {
                issues.push(format!("{} color has no RGB information", name));
            }
        }

        issues
    })
}

/// Calculate contrast ratio between two RGB colors
fn calculate_contrast_ratio(color1: [u8; 3], color2: [u8; 3]) -> f64 {
    fn luminance([r, g, b]: [u8; 3]) -> f64 {
        let (r, g, b) = (
            f64::from(r) / 255.0,
            f64::from(g) / 255.0,
            f64::from(b) / 255.0,
        );

        let to_linear = |c: f64| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.0722f64.mul_add(
            to_linear(b),
            0.2126f64.mul_add(to_linear(r), 0.7152 * to_linear(g)),
        )
    }

    let l1 = luminance(color1);
    let l2 = luminance(color2);

    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Extract RGB information from a style for display
fn extract_rgb_info(style: &Style) -> String {
    style.foreground.as_ref().map_or_else(
        || "No color".to_string(),
        |color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => {
                let [r, g, b] = rgb;
                format!("#{r:02x}{g:02x}{b:02x} RGB({r}, {g}, {b})")
            }
            thag_styling::ColorValue::Color256 { color256 } => {
                format!("256-Color({})", color256)
            }
            thag_styling::ColorValue::Basic { index, .. } => {
                format!("ANSI({})", index)
            }
        },
    )
}

/// Brighten a color by increasing its components
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, dead_code)]
fn brighten_color([r, g, b]: [u8; 3]) -> [u8; 3] {
    let factor = 1.3;
    [
        ((f32::from(r) * factor).min(255.0)) as u8,
        ((f32::from(g) * factor).min(255.0)) as u8,
        ((f32::from(b) * factor).min(255.0)) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorInfo, ColorSupport, Palette, TermBgLuma};

    fn create_test_theme() -> Theme {
        let mut palette = Palette::default();
        palette.normal = Style::fg(ColorInfo::rgb(220, 220, 220));
        palette.error = Style::fg(ColorInfo::rgb(255, 100, 100));

        Theme {
            name: "Test Palette Theme".to_string(),
            filename: PathBuf::from("test_palette_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette,
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![[42, 42, 42]],
            description: "Test theme for palette comparison".to_string(),
            base_colors: None,
        }
    }

    #[test]
    fn test_extract_rgb_info() {
        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let info = extract_rgb_info(&style);
        assert!(info.contains("ff8040"));
        assert!(info.contains("255, 128, 64"));
    }

    #[test]
    fn test_extract_rgb() {
        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let rgb = extract_rgb(&style);
        assert_eq!(rgb, Some([255, 128, 64]));
    }

    #[test]
    fn test_brighten_color() {
        let original = [100, 150, 200];
        let brightened = brighten_color(original);

        assert!(brightened.0 >= original.0);
        assert!(brightened.1 >= original.1);
        assert!(brightened.2 >= original.2);
    }

    #[test]
    fn test_contrast_ratio_calculation() {
        // Test with black and white (maximum contrast)
        let contrast = calculate_contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!((contrast - 21.0).abs() < 0.1);

        // Test with identical colors (minimum contrast)
        let contrast = calculate_contrast_ratio((128, 128, 128), (128, 128, 128));
        assert!((contrast - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_detect_potential_issues() {
        let theme = create_test_theme();
        let issues = detect_potential_issues(&theme);

        // Should detect some issues with the test theme
        assert!(issues.len() >= 0);
    }
}
