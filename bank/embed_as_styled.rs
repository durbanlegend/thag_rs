/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["color_detect"] }
*/
use thag_styling::{cprtln, Role, Styleable, Styler};
fn main() {
    let cstring1 = "Heading1 and underlined!".as_styled(Role::HD1.underline());
    let cstring2 = "Heading2 and italic!".as_styled(Role::HD2.italic());
    let embed = format!("Error {cstring1} error {cstring2} error").as_styled(Role::ERR);

    cprtln!(Role::WARN, "Warning {embed} warning");
}
