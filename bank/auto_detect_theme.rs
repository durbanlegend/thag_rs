/*[toml]
[dependencies]
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_detect", "config", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_detect", "config", "simplelog"] }
*/
use thag_rs::styling::Theme;

fn auto_detect_theme() -> Theme {
    if let Ok(bg_rgb) = termbg::rgb() {
        if let Some(theme) = Theme::detect_from_background(bg_rgb) {
            println!("Detected theme based on background color");
            theme
        } else {
            println!("Using default theme");
            Theme::default()
        }
    } else {
        println!("Could not detect background color, using default theme");
        Theme::default()
    }
}
