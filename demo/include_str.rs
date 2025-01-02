/// Simple demo of `std::include_str` macro showing how to includes other files in demo or neighboring
/// directories.
///
/// This requires a main method so that `thag` won't move the snippet to a location under temp_dir().
///
/// Not suitable for running from a URL.
//# Purpose: demo technique
//# Categories: basic, educational, technique
fn main() {
    let my_str = include_str!("hello.rs");
    assert_eq!(my_str, "//: Obligatory Hello World as a snippet\n//# Purpose: Demo Hello World snippet\n//# Categories: basic\nlet other = \"World\";\nprintln!(\"Hello, {other}!\");\n");
    print!("{my_str}");
}
