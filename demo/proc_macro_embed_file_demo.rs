#![allow(dead_code)]
/// Demonstrates embedding a single file at compile time
//# Purpose: demonstrate file embedding proc macro
//# Categories: proc_macros, filesystem
use thag_demo_proc_macros::embed_file;

// Embed the contents of hello.rs
// const HELLO_SOURCE: &str = embed_file!("/Users/donf/.config/thag_rs/config.toml");
// const HELLO_SOURCE: &str = include_str!("hello.rs");
const HELLO_SOURCE: &str = embed_file!("demo/hello.rs");

fn main() {
    println!("Contents of hello.rs:");
    println!("-------------------");
    println!("{HELLO_SOURCE}");
    println!("-------------------");
    println!("Length: {} bytes", HELLO_SOURCE.len());
}
