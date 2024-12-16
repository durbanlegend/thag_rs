/*[toml]
[dependencies]
rustc_version = "0.4.0"
[features]
nightly = ["specialization"]
specialization = []
*/
#![feature(specialization)]
// #[cfg(feature = "nightly")]
#![cfg_attr(feature != "nightly", !feature(specialization))]

use std::fmt::{Debug, Display};

trait DisplayOrDebug {
    fn format(&self) -> String;
}

default impl<T: Debug> DisplayOrDebug for T {
    fn format(&self) -> String {
        format!("{:?}", self)
    }
}

impl<T: Display + Debug> DisplayOrDebug for T {
    fn format(&self) -> String {
        format!("{}", self)
    }
}

fn print_if_displayable<T: DisplayOrDebug>(value: &T) {
    println!("{}", value.format());
}

fn main() {
    let my_string = "Hello, world!";
    let my_number = 42;

    print_if_displayable(&my_string);
    print_if_displayable(&my_number);
}
