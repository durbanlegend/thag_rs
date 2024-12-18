macro_rules! println_trimmed {
    ($val:expr) => {{
        // Format the value using Debug
        let formatted = format!("{:?}", $val);

        // Trim surrounding double quotes if present
        let trimmed = formatted.trim_matches('"');

        // Print the result using Display
        println!("{}", trimmed);
    }};
}

fn main() {
    let s = "An &str";
    let string = String::from("A String");
    let num = 42;
    let list = vec![1, 2, 3];
    let unit = ();
    let result1 = Some("&str");
    let result2 = None::<&str>;

    println_trimmed!(s); // Outputs: An &str
    println_trimmed!(num); // Outputs: 42
    println_trimmed!(list); // Outputs: [1, 2, 3]
    println_trimmed!(string); // Outputs: An &str
    println_trimmed!(unit); // Outputs: ?
    println_trimmed!(result1); // Outputs: Some(An &str)
    println_trimmed!(result2); // Outputs: None
}
