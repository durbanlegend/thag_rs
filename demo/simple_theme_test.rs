/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

//# Purpose: Simple test for runtime theme loading functionality
//# Categories: demo, styling, theming

/// Simple test to debug theme loading issues
use std::env;
use std::fs;
use thag_styling::{set_verbosity, Theme};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    set_verbosity!(debug);

    println!("🔧 Simple Theme Loading Test");
    println!("============================\n");

    // Create a test theme file
    let temp_dir = "/tmp/simple_theme_test";
    fs::create_dir_all(temp_dir)?;

    let theme_content = r##"
name = "simple-test"
description = "A simple test theme"
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

    let theme_file = format!("{}/simple-test.toml", temp_dir);
    fs::write(&theme_file, theme_content)?;
    println!("📝 Created theme file: {}", theme_file);

    // Test direct file loading
    println!("\n🧪 Test 1: Direct file loading");
    match Theme::load_from_file(std::path::Path::new(&theme_file)) {
        Ok(theme) => {
            println!("✅ Successfully loaded theme directly: {}", theme.name);
            println!("   Description: {}", theme.description);
            println!("   Is builtin: {}", theme.is_builtin);
        }
        Err(e) => {
            println!("❌ Failed to load theme directly: {}", e);
        }
    }

    // Test runtime loading with THAG_THEME_DIR
    println!("\n🧪 Test 2: Runtime loading with THAG_THEME_DIR");
    env::set_var("THAG_THEME_DIR", temp_dir);

    match Theme::get_theme_runtime_or_builtin("simple-test") {
        Ok(theme) => {
            println!("✅ Successfully loaded theme via runtime: {}", theme.name);
            println!("   Description: {}", theme.description);
            println!("   Is builtin: {}", theme.is_builtin);
        }
        Err(e) => {
            println!("❌ Failed to load theme via runtime: {}", e);
        }
    }

    // Clean up
    env::remove_var("THAG_THEME_DIR");
    fs::remove_file(&theme_file)?;
    fs::remove_dir(temp_dir)?;

    println!("\n✨ Test completed!");
    Ok(())
}
