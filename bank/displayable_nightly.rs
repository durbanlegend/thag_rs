/*[toml]
[dependencies]
rustc_version = "0.4.0"
[features]
nightly = []
specialization = ["nightly"]
*/

//: Demo of a nightly specialization that allows a trait to be
// Conditionally enable the specialization feature on nightly builds.
//
// Neither ChatGPT nor Gemini can help me get this to work.
#![cfg_attr(feature = "nightly", feature(specialization))]

// #[cfg(feature = "specialization")]
use std::fmt::{Debug, Display};

#[cfg(feature = "specialization")]
trait DisplayOrDebug {
    fn format(&self) -> String;
}

#[cfg(feature = "specialization")]
default impl<T: Debug> DisplayOrDebug for T {
    fn format(&self) -> String {
        format!("{:?}", self)
    }
}

#[cfg(feature = "specialization")]
impl<T: Display + Debug> DisplayOrDebug for T {
    fn format(&self) -> String {
        format!("{}", self)
    }
}

#[cfg(feature = "specialization")]
fn print_if_displayable<T: DisplayOrDebug>(value: &T) {
    println!("{}", value.format());
}

fn main() {
    println!("Hello world");
    #[cfg(feature = "specialization")]
    {
        let my_string = "Hello, world!";
        let my_number = 42;
        let my_option = Some("Some(option)");
        let my_unit = ();

        print_if_displayable(&my_string);
        print_if_displayable(&my_number);
        print_if_displayable(&my_option);
        print_if_displayable(&my_unit);
    }
}
