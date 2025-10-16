fn main() {
    let closure = get_line_closure();
    println!("Line: {}", closure()); // Will always print the line number from get_line_closure()

    fn get_line_closure() -> impl Fn() -> u32 {
        || line!() // Expands to the line number where this closure is defined
    }
}
