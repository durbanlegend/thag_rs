#![feature(format_args_nl)]
//: Demo how snippet processing's use of debug print ({:?}) leaves final string in double quotes.
//: If this is not happening it means I fixed it.
let who = "world";
println!("Hello {who}!");
println!("format_args_nl: {}", format_args_nl!("Greeted {who}"));
format!("Greeted {who}")
// 42
