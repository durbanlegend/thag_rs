/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
*/

/// Terminal Color Diagnostics
///
/// This script performs comprehensive diagnostics of terminal color capabilities,
/// specifically designed to investigate issues where RGB truecolor sequences
/// display incorrectly while palette-indexed colors work correctly.
///
/// The script tests multiple aspects of color handling:
/// - Basic ANSI color support
/// - 256-color palette support
/// - RGB truecolor support
/// - OSC sequence handling
/// - Terminal environment detection
/// - Color profile and gamma correction issues
//# Purpose: Comprehensive terminal color capability diagnostics and troubleshooting
//# Categories: color, debugging, diagnosis, terminal
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

/// Test color struct for diagnostics
#[derive(Debug, Clone, Copy)]
struct TestColor {
    r: u8,
    g: u8,
    b: u8,
    name: &'static str,
    expected_appearance: &'static str,
}

impl TestColor {
    const fn new(
        r: u8,
        g: u8,
        b: u8,
        name: &'static str,
        expected_appearance: &'static str,
    ) -> Self {
        Self {
            r,
            g,
            b,
            name,
            expected_appearance,
        }
    }
}

/// Diagnostic test colors chosen to highlight color handling issues
const DIAGNOSTIC_COLORS: &[TestColor] = &[
    TestColor::new(91, 116, 116, "Duck-egg Blue-green", "muted blue-green"),
    TestColor::new(255, 128, 64, "Orange", "bright orange"),
    TestColor::new(64, 192, 64, "Lime Green", "bright lime green"),
    TestColor::new(192, 64, 192, "Magenta", "bright purple/magenta"),
    TestColor::new(128, 64, 32, "Brown", "dark brown"),
    TestColor::new(32, 64, 128, "Steel Blue", "dark steel blue"),
];

/// Terminal capability flags
#[derive(Debug)]
struct TerminalCapabilities {
    basic_colors: bool,
    color_256: bool,
    truecolor_claimed: bool,
    truecolor_actual: bool,
    osc_responses: bool,
    color_profile_issue: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Terminal Color Diagnostics");
    println!("==============================");
    println!("Comprehensive analysis of terminal color handling capabilities");
    println!();

    // Phase 1: Environment Analysis
    let env_info = analyze_environment();
    display_environment_info(&env_info);

    // Phase 2: Basic Color Tests
    println!("🎨 Phase 2: Basic Color Support Tests");
    println!("======================================");
    test_basic_colors();

    // Phase 3: Advanced Color Tests
    println!("\n🌈 Phase 3: Advanced Color Tests");
    println!("=================================");
    let capabilities = test_color_capabilities();

    // Phase 4: RGB vs Palette Comparison
    println!("\n🔬 Phase 4: RGB vs Palette Color Comparison");
    println!("============================================");
    test_rgb_vs_palette_colors();

    // Phase 5: Terminal Response Tests
    println!("\n📡 Phase 5: Terminal Response Tests");
    println!("====================================");
    test_terminal_responses();

    // Phase 6: Color Profile Detection
    println!("\n🎯 Phase 6: Color Profile Analysis");
    println!("===================================");
    analyze_color_profiles();

    // Phase 7: Final Analysis and Recommendations
    println!("\n📋 Phase 7: Analysis and Recommendations");
    println!("=========================================");
    provide_recommendations(&env_info, &capabilities);

    Ok(())
}

/// Analyze terminal environment variables and settings
fn analyze_environment() -> EnvironmentInfo {
    let mut env_info = EnvironmentInfo::default();

    env_info.term = std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
    env_info.term_program = std::env::var("TERM_PROGRAM").ok();
    env_info.term_version = std::env::var("TERM_PROGRAM_VERSION").ok();
    env_info.colorterm = std::env::var("COLORTERM").ok();
    env_info.tmux = std::env::var("TMUX").is_ok();
    env_info.ssh = std::env::var("SSH_TTY").is_ok();

    env_info
}

#[derive(Debug, Default)]
struct EnvironmentInfo {
    term: String,
    term_program: Option<String>,
    term_version: Option<String>,
    colorterm: Option<String>,
    tmux: bool,
    ssh: bool,
}

fn display_environment_info(env_info: &EnvironmentInfo) {
    println!("🖥️  Phase 1: Environment Analysis");
    println!("=================================");

    println!("Terminal Type: {}", env_info.term);

    if let Some(program) = &env_info.term_program {
        println!("Terminal Program: {}", program);
        if let Some(version) = &env_info.term_version {
            println!("Version: {}", version);
        }
    }

    if let Some(colorterm) = &env_info.colorterm {
        println!("COLORTERM: {}", colorterm);
    }

    if env_info.tmux {
        println!("⚠️  Running inside tmux (may affect color handling)");
    }

    if env_info.ssh {
        println!("⚠️  SSH session detected (may affect color handling)");
    }

    // Analyze known terminal types
    match env_info.term_program.as_deref() {
        Some("zed") => {
            println!("📝 Zed Terminal detected");
            println!("   Known issues: RGB truecolor sequences may display incorrectly");
            println!("   while palette-indexed colors work correctly");
        }
        Some("iTerm.app") => {
            println!("📝 iTerm2 detected - generally good color support");
        }
        Some("Apple_Terminal") => {
            println!("📝 Apple Terminal detected - basic color support");
        }
        Some("WezTerm") => {
            println!("📝 WezTerm detected - excellent color support");
        }
        _ => {
            println!(
                "📝 Terminal type analysis: {}",
                if env_info.term.contains("256") {
                    "Likely 256-color support"
                } else if env_info.term.contains("color") {
                    "Basic color support"
                } else {
                    "Unknown color support"
                }
            );
        }
    }
    println!();
}

fn test_basic_colors() {
    println!("Testing basic 16-color ANSI support:");

    // Test basic colors (0-7)
    print!("Standard: ");
    for i in 0..8 {
        print!("\x1b[38;5;{}m█\x1b[0m", i);
    }
    println!();

    // Test bright colors (8-15)
    print!("Bright:   ");
    for i in 8..16 {
        print!("\x1b[38;5;{}m█\x1b[0m", i);
    }
    println!();

    // Test background colors
    print!("BG Test:  ");
    for i in 0..8 {
        print!("\x1b[48;5;{}m \x1b[0m", i);
    }
    println!();
}

fn test_color_capabilities() -> TerminalCapabilities {
    let mut caps = TerminalCapabilities {
        basic_colors: true, // Assume basic colors work if we got this far
        color_256: false,
        truecolor_claimed: false,
        truecolor_actual: false,
        osc_responses: false,
        color_profile_issue: false,
    };

    // Test 256-color support
    println!("Testing 256-color support:");
    print!("Color cube sample: ");
    for i in [196, 46, 21, 201, 226, 51] {
        // Red, Green, Blue, Magenta, Yellow, Cyan
        print!("\x1b[38;5;{}m██\x1b[0m", i);
    }
    println!();
    caps.color_256 = true; // If we can send the sequence, assume it works

    // Check environment claims about truecolor
    if std::env::var("COLORTERM").map_or(false, |v| v.contains("truecolor") || v.contains("24bit"))
    {
        caps.truecolor_claimed = true;
        println!("✅ Environment claims truecolor support (COLORTERM)");
    }

    // Test actual truecolor with gradient
    println!("Testing truecolor with color gradient:");
    print!("RGB gradient: ");
    for i in 0..16 {
        let red = 255 - (i * 16);
        let blue = i * 16;
        print!("\x1b[38;2;{};0;{}m█\x1b[0m", red, blue);
    }
    println!();

    // We can't easily detect if truecolor actually works without user feedback
    caps.truecolor_actual = caps.truecolor_claimed; // Conservative assumption

    caps
}

fn test_rgb_vs_palette_colors() {
    println!("Detailed RGB vs Palette comparison:");
    println!("(Look for color differences between RGB and 256-color versions)");
    println!();

    for color in DIAGNOSTIC_COLORS {
        let closest_256 = find_closest_256_color(color.r, color.g, color.b);

        println!(
            "🎨 {} RGB({}, {}, {})",
            color.name, color.r, color.g, color.b
        );
        println!("   Expected: {}", color.expected_appearance);

        // RGB version
        print!("   RGB:     ");
        print!(
            "\x1b[38;2;{};{};{}m████████████\x1b[0m",
            color.r, color.g, color.b
        );
        print!(" \x1b[48;2;{};{};{}m    \x1b[0m", color.r, color.g, color.b);
        println!(" ESC[38;2;{};{};{}m", color.r, color.g, color.b);

        // 256-color equivalent
        print!("   256-color: ");
        print!("\x1b[38;5;{}m████████████\x1b[0m", closest_256);
        print!(" \x1b[48;5;{}m    \x1b[0m", closest_256);
        println!(" ESC[38;5;{}m", closest_256);

        println!("   💡 If these look different, RGB sequences aren't working correctly");
        println!();
    }
}

fn test_terminal_responses() {
    println!("Testing terminal OSC sequence responses:");
    println!("(This may timeout if terminal doesn't support OSC queries)");

    if enable_raw_mode().is_err() {
        println!("❌ Cannot enable raw mode for response testing");
        return;
    }

    let tests = [
        ("\x1b]10;?\x07", "OSC 10", "foreground color query"),
        ("\x1b]11;?\x07", "OSC 11", "background color query"),
        ("\x1b]4;15;?\x07", "OSC 4", "palette color 15 query"),
    ];

    for (sequence, name, description) in &tests {
        println!("Testing {} ({}):", name, description);

        let mut stdout = io::stdout();
        let mut stdin = io::stdin();

        // Send query
        stdout.write_all(sequence.as_bytes()).unwrap();
        stdout.flush().unwrap();

        // Try to read response
        match read_terminal_response(&mut stdin, Duration::from_millis(300)) {
            Some(response) => {
                println!(
                    "   ✅ Response: {:?}",
                    response.chars().take(50).collect::<String>()
                );
            }
            None => {
                println!("   ❌ No response (timeout)");
            }
        }
    }

    let _ = disable_raw_mode();
    println!();
}

fn analyze_color_profiles() {
    println!("Color Profile Analysis:");
    println!("Testing for gamma correction and color space issues:");
    println!();

    // Test colors that commonly show gamma/profile issues
    let test_cases = [
        (128, 128, 128, "50% Gray"),
        (64, 64, 64, "25% Gray"),
        (192, 192, 192, "75% Gray"),
        (255, 128, 128, "Light Red"),
        (128, 255, 128, "Light Green"),
        (128, 128, 255, "Light Blue"),
    ];

    println!("Gamma test colors (should show smooth gradation):");
    for (r, g, b, name) in &test_cases {
        print!("{:>12}: ", name);
        print!("\x1b[38;2;{};{};{}m████\x1b[0m", r, g, b);
        print!(" vs 256: ");
        let idx = find_closest_256_color(*r, *g, *b);
        print!("\x1b[38;5;{}m████\x1b[0m", idx);
        println!(" RGB({},{},{}) vs idx {}", r, g, b, idx);
    }

    println!();
    println!("🔍 Look for:");
    println!("• RGB colors appearing washed out or too bright");
    println!("• Different gamma curves between RGB and palette colors");
    println!("• Color shifts (e.g., blue-green appearing salmon pink)");
}

fn provide_recommendations(env_info: &EnvironmentInfo, capabilities: &TerminalCapabilities) {
    println!("Diagnosis and Recommendations:");
    println!("==============================");

    // Specific diagnosis for common issues
    if env_info.term_program.as_deref() == Some("zed") {
        println!("🔍 DIAGNOSIS: Zed Terminal RGB Issue");
        println!("   Problem: RGB truecolor sequences display incorrectly");
        println!("   Cause: Zed may not properly handle ESC[38;2;R;G;B sequences");
        println!("   Evidence: Palette colors (ESC[38;5;N) work correctly");
        println!();
        println!("💡 WORKAROUNDS:");
        println!("   1. Use thag_styling with 256-color mode instead of truecolor");
        println!("   2. Set COLORTERM to '256color' to force palette mode:");
        println!("      export COLORTERM=256color");
        println!("   3. Use a different terminal for color-critical work");
        println!("   4. Report this issue to Zed developers");
        println!();
        println!("⚙️  THAG_STYLING CONFIGURATION:");
        println!("   Add to your environment or config:");
        println!("   export THAG_COLOR_MODE=256");
        println!("   This will force thag_styling to use palette colors");
    }

    if capabilities.truecolor_claimed && !capabilities.osc_responses {
        println!("⚠️  PARTIAL TRUECOLOR SUPPORT:");
        println!("   Your terminal claims truecolor support but doesn't respond to OSC queries");
        println!("   This may indicate incomplete color support implementation");
    }

    if env_info.tmux {
        println!("🔧 TMUX CONSIDERATIONS:");
        println!("   Tmux can interfere with color sequences");
        println!("   Try testing outside tmux to isolate issues");
        println!("   Consider tmux color configuration options");
    }

    println!("\n🛠️  GENERAL SOLUTIONS:");
    println!("=====================================");
    println!("1. Force specific color modes in thag_styling:");
    println!("   export THAG_COLOR_MODE=basic    # 16 colors only");
    println!("   export THAG_COLOR_MODE=256      # 256-color palette");
    println!("   export THAG_COLOR_MODE=truecolor # 24-bit RGB (if working)");
    println!();
    println!("2. Terminal-specific fixes:");
    println!("   • Update terminal to latest version");
    println!("   • Check terminal color profile settings");
    println!("   • Try different TERM values (xterm-256color, etc.)");
    println!();
    println!("3. Debug further with:");
    println!("   printf '\\x1b[38;2;255;0;0mRED\\x1b[0m' # Should be red");
    println!("   printf '\\x1b[38;5;196mRED\\x1b[0m'   # Should also be red");
    println!("   If colors differ, RGB sequences aren't working properly");
}

/// Read terminal response with timeout
fn read_terminal_response(stdin: &mut io::Stdin, timeout: Duration) -> Option<String> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = Instant::now();

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            Ok(1) => {
                buffer.push(temp_buffer[0]);

                // Check for terminators
                if temp_buffer[0] == 0x07
                    || (buffer.len() >= 2
                        && buffer[buffer.len() - 2] == 0x1b
                        && buffer[buffer.len() - 1] == b'\\')
                {
                    break;
                }

                if buffer.len() > 1024 {
                    break;
                }
            }
            Ok(0) => thread::sleep(Duration::from_millis(1)),
            Ok(_) => thread::sleep(Duration::from_millis(1)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => break,
        }
    }

    if buffer.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&buffer).to_string())
    }
}

/// Find closest 256-color index for RGB color
fn find_closest_256_color(r: u8, g: u8, b: u8) -> u8 {
    let mut best_index = 0u8;
    let mut best_distance = u32::MAX;

    // Basic 16 colors
    let basic_colors = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    for (i, (br, bg, bb)) in basic_colors.iter().enumerate() {
        let distance = color_distance(r, g, b, *br, *bg, *bb);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    // 216 color cube (16-231)
    for i in 16..232 {
        let cube_idx = i - 16;
        let cube_r = (cube_idx / 36) * 51;
        let cube_g = ((cube_idx % 36) / 6) * 51;
        let cube_b = (cube_idx % 6) * 51;

        let distance = color_distance(r, g, b, cube_r as u8, cube_g as u8, cube_b as u8);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    // Grayscale (232-255)
    for i in 232..=255 {
        let gray = 8 + (i - 232) * 10;
        let distance = color_distance(r, g, b, gray as u8, gray as u8, gray as u8);
        if distance < best_distance {
            best_distance = distance;
            best_index = i as u8;
        }
    }

    best_index
}

/// Calculate Manhattan distance between colors
fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    ((r1 as i32 - r2 as i32).abs() + (g1 as i32 - g2 as i32).abs() + (b1 as i32 - b2 as i32).abs())
        as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_closest_256_color() {
        assert_eq!(find_closest_256_color(255, 0, 0), 9); // Bright red
        assert_eq!(find_closest_256_color(0, 0, 0), 0); // Black
        assert_eq!(find_closest_256_color(255, 255, 255), 15); // White
    }

    #[test]
    fn test_color_distance() {
        assert_eq!(color_distance(0, 0, 0, 0, 0, 0), 0);
        assert_eq!(color_distance(255, 255, 255, 0, 0, 0), 765);
    }
}
