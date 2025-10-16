/// A very simple published example from the random name generator
/// `names`. See also `demo/hyper_name_server.rs`.
//# Purpose: Demo a simple snippet and featured crate.
//# Categories: technique
use names::{Generator, Name};

let mut generator = Generator::with_naming(Name::Numbered);
println!("Your project is: {}", generator.next().unwrap());
