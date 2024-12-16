/// Macro to attempt print for types whether or not they implement Debug.
/// Requires nightly Rust
#![feature(specialization)]

use std::fmt::Debug;

pub trait Printable {
    fn print(&self);
}

// Default implementation for all types that do not implement Debug
impl<T> Printable for T {
    default fn print(&self) {
        println!("Value does not implement Debug");
    }
}

// Specialized implementation for types that do implement Debug
impl<T: Debug> Printable for T {
    fn print(&self) {
        println!("{:#?}", self);
    }
}

macro_rules! my_dbg {
    ($val:expr) => {{
        $val.print();
        $val
    }};
}

fn main() {
    let debug_value = vec![1, 2, 3];
    let non_debug_value = "Hello, world!";

    my_dbg!(debug_value);
    my_dbg!(non_debug_value);
}
