/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["color_detect", "core", "simplelog"] }

[features]
color_detect = ["thag_rs/color_detect"]
default = ["color_detect"]
*/
use thag_rs::cprtln;
use thag_rs::styling::Role;
use thag_rs::Verbosity;

fn main() {
    let details = "todos los detalles";
    // thag_rs::cvlog_error!(Verbosity::Normal, "Detailed info: {}", details);
    // thag_rs::cvprtln!(Role::Info, Verbosity::N, "Detailed info: {}", details);
    cprtln!(Role::Info, Verbosity::Normal, "Detailed info: {}", details);
}
