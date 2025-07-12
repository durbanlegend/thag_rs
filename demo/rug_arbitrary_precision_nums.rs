/// Published example from the `rug` crate, showcasing the use of the crate. I added the
/// last line to return a tuple of the state of the values of interest, as a quick way
/// of displaying them.
///
/// Won't work with default Windows 11 because of `rug` crate.
/// On Linux you may need to install the m4 package.
///
//# Purpose: Demo featured crate, also how we can often run an incomplete snippet "as is".
//# Categories: crates, technique
use rug::{Assign, Integer};
let mut int = Integer::new();
assert_eq!(int, 0);
int.assign(14);
assert_eq!(int, 14);

let decimal = "98_765_432_109_876_543_210";
int.assign(Integer::parse(decimal).unwrap());
assert!(int > 100_000_000);

let hex_160 = "ffff0000ffff0000ffff0000ffff0000ffff0000";
int.assign(Integer::parse_radix(hex_160, 16).unwrap());
assert_eq!(int.significant_bits(), 160);
int = (int >> 128) - 1;
assert_eq!(int, 0xfffe_ffff_u32);

(int, decimal, hex_160)
