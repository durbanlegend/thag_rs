/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.1", features = ["derive_serde_style"] }
strum = { version = "0.26.3", features = ["derive"] }
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_detect", "core", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_detect", "core", "simplelog"] }
*/
use thag_rs::ThagError;
use thag_rs::styling::Theme;
let theme = Theme::load_builtin("dracula")?;
