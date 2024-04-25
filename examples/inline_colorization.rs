/*[toml]
[dependencies]
inline_colorization = "0.1.6"
*/

use inline_colorization::{color_red, color_reset, style_reset, style_underline};

println!("Lets the user {color_red}colorize{color_reset} and {style_underline}style the output{style_reset} text using inline variables");
