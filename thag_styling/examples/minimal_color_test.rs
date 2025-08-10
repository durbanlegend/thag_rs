//! Minimal color detection test to isolate the issue
//!
//! This test bypasses all caching and state to directly test color detection.
//!
//! Run with:
//! ```bash
//! cargo run --example minimal_color_test --features "color_detect"
//! ```

use std::env;

fn main() {
    println!("🧪 Minimal Color Detection Test\n");

    // Check environment that might affect detection
    println!("1. Environment Check:");
    if env::var("TEST_ENV").is_ok() {
        println!("   ⚠️ TEST_ENV is set - this forces fallback!");
        env::remove_var("TEST_ENV");
        println!("   Removed TEST_ENV for clean test");
    } else {
        println!("   ✅ TEST_ENV not set");
    }

    // Test supports_color directly
    println!("\n2. Direct supports_color test:");
    #[cfg(feature = "color_detect")]
    {
        use supports_color::{on, Stream};

        match on(Stream::Stdout) {
            Some(level) => {
                println!("   ✅ Color support detected:");
                println!("      has_basic: {}", level.has_basic);
                println!("      has_256: {}", level.has_256);
                println!("      has_16m: {}", level.has_16m);
            }
            None => {
                println!("   ❌ No color support detected");
            }
        }
    }

    // Test termbg directly
    println!("\n3. Direct termbg test:");
    #[cfg(feature = "color_detect")]
    {
        match termbg::rgb() {
            Ok(rgb) => {
                println!("   ✅ Background RGB detected: {:?}", rgb);
                let hex = format!("{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2);
                println!("   Background hex: #{}", hex);

                let is_light =
                    rgb.0 as f64 * 0.299 + rgb.1 as f64 * 0.587 + rgb.2 as f64 * 0.114 > 127.5;
                println!("   Is light: {}", is_light);
            }
            Err(e) => {
                println!("   ❌ Background detection failed: {:?}", e);
            }
        }
    }

    // Test raw mode check directly
    println!("\n4. Direct raw mode test:");
    #[cfg(feature = "color_detect")]
    {
        match ratatui::crossterm::terminal::is_raw_mode_enabled() {
            Ok(raw_mode) => {
                println!("   ✅ Raw mode status: {}", raw_mode);
            }
            Err(e) => {
                println!("   ⚠️ Raw mode check failed: {:?}", e);
                println!("   This would cause fallback to (0,0,0)");
            }
        }
    }

    // Test thag_common detection ONCE
    println!("\n5. Single thag_common detection:");
    #[cfg(feature = "color_detect")]
    {
        let (color_support, bg_rgb) = thag_common::terminal::detect_term_capabilities();
        println!("   Color support: {:?}", color_support);
        println!("   Background RGB: {:?}", bg_rgb);
        let hex = format!("{:02x}{:02x}{:02x}", bg_rgb.0, bg_rgb.1, bg_rgb.2);
        println!("   Background hex: #{}", hex);
    }

    println!("\n✅ Test complete!");
}
