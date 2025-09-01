/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
crossterm = "0.28"
*/

//! Simple OSC 4 Test
//!
//! A minimal test script to debug OSC 4 response capture issues.
//! This script sends a single OSC 4 query and tries different methods
//! to capture the response, helping identify why responses are visible
//! but not being captured programmatically.

//# Purpose: Debug OSC 4 response capture mechanisms
//# Categories: terminal, debugging, experimental

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Try to parse OSC 4 response
fn parse_osc4_response(response: &str, expected_index: u8) -> Option<Rgb> {
    let pattern = format!("4;{};", expected_index);

    if let Some(start_pos) = response.find(&pattern) {
        let response_part = &response[start_pos..];

        // Handle rgb: format
        if let Some(rgb_pos) = response_part.find("rgb:") {
            let rgb_data = &response_part[rgb_pos + 4..];
            let parts: Vec<&str> = rgb_data.split('/').collect();

            if parts.len() >= 3 {
                // Parse first component to see what we get
                let r_str = parts[0]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect::<String>();
                let g_str = parts[1]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect::<String>();
                let b_str = parts[2]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect::<String>();

                if let (Ok(r_val), Ok(g_val), Ok(b_val)) = (
                    u16::from_str_radix(&r_str, 16),
                    u16::from_str_radix(&g_str, 16),
                    u16::from_str_radix(&b_str, 16),
                ) {
                    // Convert 16-bit to 8-bit by taking high byte
                    let r = (r_val >> 8) as u8;
                    let g = (g_val >> 8) as u8;
                    let b = (b_val >> 8) as u8;

                    return Some(Rgb::new(r, g, b));
                }
            }
        }
    }

    None
}

/// Method 1: Direct stdin reading with crossterm raw mode
fn test_crossterm_stdin() -> Option<Rgb> {
    println!("🧪 Testing crossterm stdin method...");

    // Enable raw mode
    if enable_raw_mode().is_err() {
        println!("❌ Could not enable raw mode");
        return None;
    }

    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    // Send query for color 0
    print!("\x1b]4;0;?\x07");
    stdout.flush().unwrap();

    // Try to read response
    let mut buffer = Vec::new();
    let mut temp_buffer = [0u8; 1];
    let start = Instant::now();
    let timeout = Duration::from_millis(200);

    while start.elapsed() < timeout {
        match stdin.read(&mut temp_buffer) {
            // Ok(n) if n > 0 => {
            Ok(1..) => {
                buffer.push(temp_buffer[0]);

                // Check if we have enough for a response
                if buffer.len() > 10 {
                    let response = String::from_utf8_lossy(&buffer);
                    println!("   Raw buffer: {:?}", response);

                    if let Some(rgb) = parse_osc4_response(&response, 0) {
                        disable_raw_mode().unwrap();
                        return Some(rgb);
                    }
                }

                if buffer.len() > 100 {
                    break;
                }
            }
            Ok(0) => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(1));
            }
            Err(_) => break,
        }
    }

    disable_raw_mode().unwrap();

    if !buffer.is_empty() {
        let response = String::from_utf8_lossy(&buffer);
        println!("   Final buffer: {:?}", response);
    } else {
        println!("   No data captured");
    }

    None
}

/// Method 2: Try using shell command with script/expect
fn test_shell_method() -> Option<Rgb> {
    println!("🧪 Testing shell script method...");

    // Create a shell command that sends OSC query and captures output
    let script = r#"
        # Send OSC 4 query
        printf '\e]4;0;?\a'
        # Try to read response with timeout
        read -t 1 response
        echo "RESPONSE: $response"
    "#;

    match Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);

            println!("   Stdout: {:?}", stdout_str);
            println!("   Stderr: {:?}", stderr_str);

            // Try to parse from either output
            if let Some(rgb) = parse_osc4_response(&stdout_str, 0) {
                return Some(rgb);
            }
            if let Some(rgb) = parse_osc4_response(&stderr_str, 0) {
                return Some(rgb);
            }
        }
        Err(e) => {
            println!("   Shell command failed: {}", e);
        }
    }

    None
}

/// Method 3: Use expect-like approach
fn test_expect_method() -> Option<Rgb> {
    println!("🧪 Testing expect-style method...");

    // Try using unbuffer or expect if available
    let commands = [
        (
            "unbuffer",
            vec!["sh", "-c", "printf '\\e]4;0;?\\a'; sleep 0.1"],
        ),
        (
            "expect",
            vec![
                "-c",
                "send \"\\033]4;0;?\\007\"; expect -timeout 1 -re {.*}; puts $expect_out(buffer)",
            ],
        ),
    ];

    for (cmd_name, args) in commands {
        match Command::new(cmd_name)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(output) => {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                let stderr_str = String::from_utf8_lossy(&output.stderr);

                println!("   {} stdout: {:?}", cmd_name, stdout_str);
                println!("   {} stderr: {:?}", cmd_name, stderr_str);

                if let Some(rgb) = parse_osc4_response(&stdout_str, 0) {
                    return Some(rgb);
                }
                if let Some(rgb) = parse_osc4_response(&stderr_str, 0) {
                    return Some(rgb);
                }
            }
            Err(_) => {
                println!("   {} not available", cmd_name);
            }
        }
    }

    None
}

/// Method 4: Manual observation test
fn test_manual_observation() {
    println!("🧪 Testing manual observation...");
    println!("   Watch carefully for escape sequences after this query:");
    print!("\x1b]4;0;?\x07");
    io::stdout().flush().unwrap();

    // Give time for response to appear
    thread::sleep(Duration::from_millis(500));
    println!("   (Did you see any escape sequences above?)");
}

/// Method 5: Redirect to file test
fn test_file_redirect() -> Option<Rgb> {
    println!("🧪 Testing file redirection method...");

    let script = r#"
        # Redirect all output to a temp file
        exec 3>&1 4>&2
        exec 1>/tmp/osc4_test.out 2>&1

        # Send query
        printf '\e]4;0;?\a'

        # Wait briefly
        sleep 0.2

        # Restore output
        exec 1>&3 2>&4

        # Show what we captured
        cat /tmp/osc4_test.out
        rm -f /tmp/osc4_test.out
    "#;

    match Command::new("sh").arg("-c").arg(script).output() {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            println!("   File capture result: {:?}", result);

            if let Some(rgb) = parse_osc4_response(&result, 0) {
                return Some(rgb);
            }
        }
        Err(e) => {
            println!("   File redirect failed: {}", e);
        }
    }

    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Simple OSC 4 Response Capture Test");
    println!("====================================");
    println!("This script tests different methods to capture OSC 4 responses.");
    println!("The goal is to understand why responses are visible but not captured.");
    println!();

    println!(
        "🖥️  Terminal: {:?}",
        std::env::var("TERM").unwrap_or_default()
    );
    println!(
        "🎨 Program: {:?}",
        std::env::var("TERM_PROGRAM").unwrap_or_default()
    );
    println!();

    let methods = [
        (
            "crossterm stdin",
            test_crossterm_stdin as fn() -> Option<Rgb>,
        ),
        ("shell script", test_shell_method),
        ("expect-style", test_expect_method),
        ("file redirect", test_file_redirect),
    ];

    let mut successful_methods = Vec::new();

    for (name, test_fn) in methods {
        println!("Testing: {}", name);
        match test_fn() {
            Some(rgb) => {
                println!("✅ SUCCESS: RGB({}, {}, {})", rgb.r, rgb.g, rgb.b);
                successful_methods.push(name);
            }
            None => {
                println!("❌ Failed to capture response");
            }
        }
        println!();
    }

    // Manual observation test (always run)
    test_manual_observation();
    println!();

    println!("📊 Results Summary:");
    println!("==================");

    if successful_methods.is_empty() {
        println!("❌ No methods successfully captured OSC 4 responses");
        println!();
        println!("💡 This suggests that:");
        println!("   • OSC responses may go to a different output stream");
        println!("   • Terminal may be buffering or filtering responses");
        println!("   • Responses might need different capture timing");
        println!("   • Terminal-specific handling may be required");
    } else {
        println!("✅ Successful methods:");
        for method in successful_methods {
            println!("   • {}", method);
        }
    }

    println!();
    println!("🔍 Next steps:");
    println!("   • Try running in different terminals");
    println!("   • Check if responses appear visually but aren't captured");
    println!("   • Consider terminal-specific libraries");
    println!("   • Test with different OSC sequences");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_osc4_response() {
        let response = "4;0;rgb:1818/1818/1818";
        let result = parse_osc4_response(response, 0);
        assert_eq!(result, Some(Rgb::new(24, 24, 24)));
    }
}
