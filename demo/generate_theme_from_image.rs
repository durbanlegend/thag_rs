/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["full"] }
*/
/// Demo of generating a `thag_styling` theme from an image.
//# Purpose: Demo making your own themes
//# Categories: color, styling, technique, xterm

use thag_styling::{display_theme_roles, save_theme_to_file, ImageThemeGenerator};

let generator = ImageThemeGenerator::new();
let image_path_str = "/Users/donf/projects/thag_rs/assets/raphael-school-of-athens.png";
// let image_path_str = "/Users/donf/projects/thag_rs/assets/botticelli-birth-of-venus.png";
// let image_path_str = "/Users/donf/projects/thag_rs/assets/munch-the-scream.png";
// let image_path_str = "/Users/donf/projects/thag_rs/monet-woman-with-parasol.png";
eprintln!("Generating from image {image_path_str}");
let theme = generator.generate_from_file(image_path_str)?;

// println!("{theme:#?}");
display_theme_roles(&theme);

save_theme_to_file(&theme, "thag_styling/themes/built_in/raphael-school-of-athens.toml")?;
// save_theme_to_file(&theme, "thag_styling/themes/built_in/botticelli-birth-of-venus.toml")?;
// save_theme_to_file(&theme, "thag_styling/themes/built_in/thag-munch-the-scream.toml")?;
// save_theme_to_file(&theme, "thag_styling/themes/built_in/thag-monet-woman-with-parasol.toml")?;
