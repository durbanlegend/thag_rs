/*[toml]
[dependencies]
names = { version = "0.14.0", default-features = false }
*/

use names::{Generator, Name};

let mut generator = Generator::with_naming(Name::Numbered);
println!("Your project is: {}", generator.next().unwrap());
