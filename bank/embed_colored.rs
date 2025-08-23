use colored::{Colorize, ColoredString};

/*
 * This example use colored strings in a nested way (at line 14). It shows that colored is able to
 * keep the correct color on the “!lalalala” part.
 */

fn main() {
let cstring1: ColoredString = "Bold and Red!".bold().red();
let cstring2: ColoredString = "Italic and Blue!".italic().blue();
let embed = format!("Magenta {cstring1} magenta {cstring2} magenta").magenta();

println!("Normal {embed} normal");
}
