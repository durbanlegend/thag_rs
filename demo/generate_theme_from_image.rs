/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["full"] }
*/
/// Demo of generating a `thag_styling` theme from an image.
//# Purpose: Demo making your own themes
//# Categories: color, styling, technique, xterm

use thag_styling::{ImageThemeGenerator,save_theme_to_file};

let generator = ImageThemeGenerator::new();
let image_path_str = "/Users/donf/projects/thag_rs/assets/Munch_The_Scream.png";
// let image_path_str = "PastedGraphic-1.png";
eprintln!("Generating from image {image_path_str}");
let theme = generator.generate_from_file(image_path_str)?;

println!("{theme:#?}");

save_theme_to_file(&theme, "thag_styling/themes/built_in/my_theme.toml")?;
