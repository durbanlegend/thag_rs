use std::fmt::{self, Debug, Display};

struct DisplayWrapper<T> {
    value: T,
}

impl<T: Debug> Display for DisplayWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the value using Debug
        let formatted = format!("{:?}", &self.value);

        // Trim surrounding double quotes if present
        let trimmed = formatted.trim_matches('"');

        // Print the result using Display
        write!(f, "{}", trimmed)
    }
}

fn main() {
    let wrapped_str = DisplayWrapper {
        value: "Hello, world!",
    };
    let wrapped_int = DisplayWrapper { value: 42 };

    println!("{}", wrapped_str); // Outputs: Hello, world!
    println!("{}", wrapped_int); // Outputs: 42
}
