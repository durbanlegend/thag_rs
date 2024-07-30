/*[toml]
[dependencies]
inline_colorization = "0.1.6"
*/

/// Published simple example from `inline_colorization` crate. Simple effective inline
/// styling option for text messages.
//# Purpose: Demo featured crate, also how we can often run an incomplete snippet "as is".
use inline_colorization::{color_red, color_reset, style_reset, style_underline};

println!("Lets the user {color_red}colorize{color_reset} and {style_underline}style the output{style_reset} text using inline variables");
