#![allow(unused_imports, unused_macros, unused_variables, dead_code)]
#![feature(duration_constructors)]
use std::error::Error;

// Wrapped snippet in main method to make it a program
fn main() -> Result<(), Box<dyn Error>> {
    println!("{:#?}", { std::time::Duration::from_days(10) });

    Ok(())
}
