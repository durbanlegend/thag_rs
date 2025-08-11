/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["full"] }
*/
/// Demo of generating a `thag_styling` theme from an image.
//# Purpose: Demo making your own themes
//# Categories: color, styling, technique, xterm

use thag_styling::{ImageThemeGenerator,save_theme_to_file};

let generator = ImageThemeGenerator::new();
let theme = generator.generate_from_file("PastedGraphic-1.png")?;

println!("{theme:#?}");

save_theme_to_file(&theme, "path/to/my/themes/my_theme.toml")?;
