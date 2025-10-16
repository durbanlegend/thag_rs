/// Minimal for profiling test
//# Purpose: Demo Hello World as a program

struct Foo;

impl Foo {
    fn new() -> Self {
        Foo {}
    }
}

fn bar(baz: &str) {
    println!("Hello, {baz}!");
}

fn main() {
    let other = "World 🌍";
    bar(&other);
}
