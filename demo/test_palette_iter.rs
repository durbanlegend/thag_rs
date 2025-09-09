/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
*/

//# Purpose: Test the new iter method on Palette
//# Categories: testing, development

/// Test script to verify the new iter method on Palette works correctly
use thag_styling::{Palette, PaletteConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a default palette config
    let config = PaletteConfig::default();

    // Create a palette from the config
    let palette = Palette::from_config(&config)?;

    println!("Testing new iter() method on Palette:");
    println!("=====================================");

    // Test the new iter method
    for (style_name, style) in palette.iter() {
        println!("Style: {} -> {:?}", style_name, style);
    }

    println!("\nTesting iter_mut() method for comparison:");
    println!("=========================================");

    let mut palette_mut = palette;
    let mut count = 0;
    for style in palette_mut.iter_mut() {
        count += 1;
        println!("Style #{}: {:?}", count, style);
    }

    println!("\nTest completed successfully!");
    Ok(())
}
