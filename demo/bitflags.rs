/*[toml]
[dependencies]
bitflags = "2.5.0"
*/

//# Purpose: Explore use of `bitflags` to control processing.
//# Crates: `bitflags`
//# Target: all
//# Type: program

use bitflags::bitflags;
use std::error::Error;
use std::fmt;

bitflags! {
    #[derive(Clone, PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const FORCE = 4;
    }
}

impl fmt::Display for ProcFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

fn print_flag(proc_flag: ProcFlags) {
    println!("proc_flag={proc_flag}");
}

/// Try out the `bitflags` crate.
fn main() -> Result<(), Box<dyn Error>> {
    print_flag(ProcFlags::from_bits(5).unwrap());

    println!("FORCE bits={}", ProcFlags::bits(&ProcFlags::FORCE));

    Ok(())
}
