#![allow(clippy::uninlined_format_args)]

use std::error::Error;
use thag_rs::{execute, get_args};

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = get_args();
    execute(args)?;

    Ok(())
}
