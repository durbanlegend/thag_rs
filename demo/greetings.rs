#![feature(format_args_nl)]
//: Demo how snippet processing's use of debug print ({:?}) leaves final string in double quotes.
//: If this is not happening it means I fixed it.
let user = std::env::var("USER")?;
println!("Hello {user}!");
format!("Greeted {user}")
// 42
