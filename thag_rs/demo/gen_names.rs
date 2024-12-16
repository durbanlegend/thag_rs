/*[toml]
[dependencies]
names = { version = "0.14.0", default-features = false }
*/

/// A very simple published example from the random name generator
/// `names`.
//# Purpose: Demo a simple snippet and featured crate.
//# Categories: technique
use names::{Generator, Name};

let mut generator = Generator::with_naming(Name::Numbered);
println!("Your project is: {}", generator.next().unwrap());
