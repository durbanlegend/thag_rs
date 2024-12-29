fn main() {
    let my_str = include_str!("hello.rs");
    assert_eq!(my_str, "//: Obligatory Hello World as a snippet\n//# Purpose: Demo Hello World snippet\n//# Categories: basic\nlet other = \"World\";\nprintln!(\"Hello, {other}!\");\n");
    print!("{my_str}");
}
