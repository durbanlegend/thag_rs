/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
thag_common = { version = "0.2, thag-auto" }
*/

//# Purpose: Test runtime theme loading from user-specified directories
//# Categories: demo, styling, theming

/// Demo script that tests the new runtime theme loading logic.
///
/// This script demonstrates:
/// 1. Loading themes from user-specified directories via config
/// 2. Loading themes via THAG_THEME_DIR environment variable
/// 3. Fallback to built-in themes when user themes aren't found
/// 4. Proper error handling for missing directories/themes
use std::env;
use std::fs;

use thag_common::ColorSupport;
use thag_styling::{set_verbosity, ColorInitStrategy, TermAttributes, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_verbosity!(verbose);

    println!("🎨 Testing Runtime Theme Loading");
    println!("================================\n");

    // Test 1: Try to load a built-in theme (should work)
    println!("Test 1: Loading built-in theme 'dracula'");
    match Theme::get_theme_runtime_or_builtin("dracula") {
        Ok(theme) => {
            println!("✅ Successfully loaded built-in theme: {}", theme.name);
            println!("   Description: {}", theme.description);
        }
        Err(e) => println!("❌ Failed to load dracula theme: {}", e),
    }
    println!();

    // Test 2: Test THAG_THEME_DIR environment variable
    println!("Test 2: Testing THAG_THEME_DIR environment variable");

    // Create a temporary theme directory for testing
    let temp_dir = "/tmp/test_thag_themes";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        println!("⚠️  Could not create temp directory {}: {}", temp_dir, e);
    } else {
        // Create a sample theme file
        let theme_content = r##"
name = "test-custom-theme"
description = "A custom test theme"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#1e1e1e", "#2d2d2d"]

[palette.heading1]
rgb = [255, 107, 107]

[palette.heading2]
rgb = [78, 205, 196]

[palette.heading3]
rgb = [69, 183, 209]

[palette.error]
rgb = [255, 85, 85]

[palette.warning]
rgb = [255, 184, 108]

[palette.success]
rgb = [80, 250, 123]

[palette.info]
rgb = [139, 233, 253]

[palette.emphasis]
rgb = [255, 121, 198]

[palette.code]
rgb = [241, 250, 140]

[palette.normal]
rgb = [248, 248, 242]

[palette.subtle]
rgb = [98, 114, 164]

[palette.hint]
rgb = [98, 114, 164]

[palette.debug]
rgb = [68, 71, 90]

[palette.link]
rgb = [139, 233, 253]

[palette.quote]
rgb = [241, 250, 140]

[palette.commentary]
rgb = [98, 114, 164]
"##;

        let theme_file = format!("{}/test-custom-theme.toml", temp_dir);
        if let Err(e) = fs::write(&theme_file, theme_content) {
            println!("⚠️  Could not write test theme file: {}", e);
        } else {
            println!("📁 Created test theme at: {}", theme_file);

            // Set THAG_THEME_DIR and test loading
            env::set_var("THAG_THEME_DIR", temp_dir);
            println!("🔧 Set THAG_THEME_DIR to: {}", temp_dir);

            match Theme::get_theme_runtime_or_builtin("test-custom-theme") {
                Ok(theme) => {
                    println!("✅ Successfully loaded custom theme: {}", theme.name);
                    println!("   From directory: {}", temp_dir);
                    println!("   Description: {}", theme.description);
                    println!("   Is builtin: {}", theme.is_builtin);
                }
                Err(e) => println!("❌ Failed to load custom theme: {}", e),
            }

            // Test loading with color support conversion
            println!("\n   Testing color support conversion...");
            match Theme::get_theme_runtime_or_builtin_with_color_support(
                "test-custom-theme",
                ColorSupport::Color256,
            ) {
                Ok(theme) => {
                    println!("✅ Successfully loaded custom theme with Color256 support");
                    println!("   Theme color support: {:?}", theme.min_color_support);
                }
                Err(e) => println!("❌ Failed to load custom theme with color support: {}", e),
            }

            // Clean up
            let _ = fs::remove_file(&theme_file);
            let _ = fs::remove_dir(temp_dir);
            env::remove_var("THAG_THEME_DIR");
        }
    }
    println!();

    // Test 3: Test fallback to built-in when custom theme not found
    println!("Test 3: Testing fallback to built-in themes");
    env::set_var("THAG_THEME_DIR", "/nonexistent/directory");

    match Theme::get_theme_runtime_or_builtin("dracula") {
        Ok(theme) => {
            println!(
                "✅ Successfully fell back to built-in theme: {}",
                theme.name
            );
            println!("   Is builtin: {}", theme.is_builtin);
        }
        Err(e) => println!("❌ Failed to load fallback theme: {}", e),
    }

    env::remove_var("THAG_THEME_DIR");
    println!();

    // Test 4: Test various filename patterns
    println!("Test 4: Testing filename pattern detection");
    let temp_dir = "/tmp/test_thag_theme_patterns";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        println!("⚠️  Could not create temp directory: {}", e);
    } else {
        let test_theme = r##"
name = "pattern-test"
description = "Testing filename patterns"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#1e1e1e"]

[palette.normal]
rgb = [248, 248, 242]

[palette.error]
rgb = [255, 85, 85]

[palette.success]
rgb = [80, 250, 123]

[palette.warning]
rgb = [255, 184, 108]

[palette.info]
rgb = [139, 233, 253]

[palette.emphasis]
rgb = [255, 121, 198]

[palette.code]
rgb = [241, 250, 140]

[palette.subtle]
rgb = [98, 114, 164]

[palette.hint]
rgb = [98, 114, 164]

[palette.debug]
rgb = [68, 71, 90]

[palette.link]
rgb = [139, 233, 253]

[palette.quote]
rgb = [241, 250, 140]

[palette.commentary]
rgb = [98, 114, 164]

[palette.heading1]
rgb = [255, 107, 107]

[palette.heading2]
rgb = [78, 205, 196]

[palette.heading3]
rgb = [69, 183, 209]
"##;

        // Test different filename patterns
        let patterns = vec![
            ("simple.toml", "simple"),
            ("thag-prefixed.toml", "prefixed"),
            ("thag-with-variant-light.toml", "with-variant"),
            ("thag-with-variant-dark.toml", "with-variant"),
        ];

        env::set_var("THAG_THEME_DIR", temp_dir);

        for (filename, theme_name) in patterns {
            let theme_path = format!("{}/{}", temp_dir, filename);
            let _ = fs::write(&theme_path, test_theme);

            match Theme::get_theme_runtime_or_builtin(theme_name) {
                Ok(_theme) => {
                    println!(
                        "✅ Found theme '{}' using pattern: {}",
                        theme_name, filename
                    );
                }
                Err(_) => {
                    println!(
                        "❌ Could not find theme '{}' with pattern: {}",
                        theme_name, filename
                    );
                }
            }

            let _ = fs::remove_file(&theme_path);
        }

        let _ = fs::remove_dir(temp_dir);
        env::remove_var("THAG_THEME_DIR");
    }
    println!();

    // Test 5: Test integration with TermAttributes
    println!("Test 5: Testing integration with TermAttributes initialization");

    // Test that THAG_THEME env var works with runtime loading
    let temp_dir = "/tmp/test_term_attrs";
    if fs::create_dir_all(temp_dir).is_ok() {
        let theme_content = r##"
name = "integration-test"
description = "Testing TermAttributes integration"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#282828"]

[palette.normal]
rgb = [235, 219, 178]

[palette.error]
rgb = [251, 73, 52]

[palette.success]
rgb = [184, 187, 38]

[palette.warning]
rgb = [250, 189, 47]

[palette.info]
rgb = [131, 165, 152]

[palette.emphasis]
rgb = [211, 134, 155]

[palette.code]
rgb = [142, 192, 124]

[palette.subtle]
rgb = [168, 153, 132]

[palette.hint]
rgb = [168, 153, 132]

[palette.debug]
rgb = [102, 92, 84]

[palette.link]
rgb = [131, 165, 152]

[palette.quote]
rgb = [142, 192, 124]

[palette.commentary]
rgb = [168, 153, 132]

[palette.heading1]
rgb = [235, 203, 139]

[palette.heading2]
rgb = [142, 192, 124]

[palette.heading3]
rgb = [131, 165, 152]
"##;

        let theme_file = format!("{}/integration-test.toml", temp_dir);
        if fs::write(&theme_file, theme_content).is_ok() {
            env::set_var("THAG_THEME_DIR", temp_dir);
            env::set_var("THAG_THEME", "integration-test");

            // This should use our custom theme through the normal initialization flow
            let attrs = TermAttributes::get_or_init_with_strategy(&ColorInitStrategy::Match);

            println!(
                "✅ TermAttributes initialized with theme: {}",
                attrs.theme.name
            );
            println!("   Theme is builtin: {}", attrs.theme.is_builtin);
            println!("   Theme description: {}", attrs.theme.description);

            let _ = fs::remove_file(&theme_file);
            env::remove_var("THAG_THEME");
            env::remove_var("THAG_THEME_DIR");
        }

        let _ = fs::remove_dir(temp_dir);
    }

    // Test 6: Test THAG_THEME environment variable with custom theme
    println!("\nTest 6: Testing THAG_THEME environment variable with custom theme");
    let temp_dir = "/tmp/test_thag_theme_env";
    if fs::create_dir_all(temp_dir).is_ok() {
        let theme_content = r##"
name = "env-test-theme"
description = "Testing THAG_THEME environment variable"
term_bg_luma = "dark"
min_color_support = "true_color"
backgrounds = ["#2b2b2b"]

[palette.normal]
rgb = [200, 200, 200]

[palette.error]
rgb = [255, 100, 100]

[palette.success]
rgb = [100, 255, 100]

[palette.warning]
rgb = [255, 200, 100]

[palette.info]
rgb = [100, 200, 255]

[palette.emphasis]
rgb = [255, 150, 200]

[palette.code]
rgb = [200, 255, 150]

[palette.subtle]
rgb = [120, 120, 120]

[palette.hint]
rgb = [120, 120, 120]

[palette.debug]
rgb = [80, 80, 80]

[palette.link]
rgb = [100, 200, 255]

[palette.quote]
rgb = [200, 255, 150]

[palette.commentary]
rgb = [120, 120, 120]

[palette.heading1]
rgb = [255, 120, 120]

[palette.heading2]
rgb = [120, 255, 120]

[palette.heading3]
rgb = [120, 120, 255]
"##;

        let theme_file = format!("{}/env-test-theme.toml", temp_dir);
        if fs::write(&theme_file, theme_content).is_ok() {
            env::set_var("THAG_THEME_DIR", temp_dir);
            env::set_var("THAG_THEME", "env-test-theme");
            println!(
                "🔧 Set THAG_THEME_DIR={} and THAG_THEME=env-test-theme",
                temp_dir
            );

            match Theme::get_theme_runtime_or_builtin("env-test-theme") {
                Ok(theme) => {
                    println!(
                        "✅ Successfully loaded theme via THAG_THEME: {}",
                        theme.name
                    );
                    println!("   Description: {}", theme.description);
                    println!("   Is builtin: {}", theme.is_builtin);
                }
                Err(e) => {
                    println!("❌ Failed to load theme via THAG_THEME: {}", e);
                }
            }

            let _ = fs::remove_file(&theme_file);
            env::remove_var("THAG_THEME");
            env::remove_var("THAG_THEME_DIR");
        }

        let _ = fs::remove_dir(temp_dir);
    }

    println!("\n🎉 Runtime theme loading tests completed!");
    println!("\n💡 To use this feature:");
    println!("   1. Set THAG_THEME_DIR=/path/to/your/themes");
    println!("   2. Or add theme_dir = \"/path/to/your/themes\" to your thag config");
    println!("   3. Place .toml theme files in that directory");
    println!("   4. Use THAG_THEME=your-theme-name to select them");

    Ok(())
}
