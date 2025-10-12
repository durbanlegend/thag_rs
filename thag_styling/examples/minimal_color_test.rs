//! Minimal color detection test to isolate the issue
//!
//! This test bypasses all caching and state to directly test color detection.
//!
//! Run with:
//! ```bash
//! cargo run -p thag_styling --example minimal_color_test --features "color_detect"
//! ```

#![allow(clippy::suboptimal_flops)]
use std::env;
use std::time::Duration;

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
        match termbg::rgb(Duration::from_millis(500)) {
            Ok(rgb) => {
                let (r, g, b) = (rgb.r, rgb.g, rgb.b);
                println!("   ✅ Background RGB detected: {:?}", rgb);
                let hex = format!("{:02x}{:02x}{:02x}", r, g, b);
                println!("   Background hex: #{}", hex);

                let is_light =
                    f64::from(r) * 0.299 + f64::from(g) * 0.587 + f64::from(b) * 0.114 > 127.5;
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
        let [r, g, b] = bg_rgb;
        println!("   Color support: {:?}", color_support);
        println!("   Background RGB: {:?}", bg_rgb);
        let hex = format!("{r:02x}{g:02x}{b:02x}");
        println!("   Background hex: #{}", hex);
    }

    println!("\n✅ Test complete!");
}
