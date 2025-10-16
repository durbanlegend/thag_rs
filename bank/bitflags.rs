use bitflags::{
    bitflags,
    parser::{ParseError, ParseHex, WriteHex},
};
use std::str::FromStr;

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Flags: u32 {
        /// The value `A`, at bit position `0`.
        const A = 0b00000001;
        /// The value `B`, at bit position `1`.
        const B = 0b00000010;
        /// The value `C`, at bit position `2`.
        const C = 0b00000100;

        /// The combination of `A`, `B`, and `C`.
        const ABC = Self::A.bits() | Self::B.bits() | Self::C.bits();
    }
}

impl FromStr for Flags {
    type Err = bitflags::parser::ParseError;

    fn from_str(flags: &str) -> Result<Self, Self::Err> {
        bitflags::parser::from_str(flags)
    }
}

fn main() {
    let e1 = Flags::A | Flags::C;
    let e2 = Flags::B | Flags::C;
    assert_eq!((e1 | e2), Flags::ABC); // union
    assert_eq!((e1 & e2), Flags::C); // intersection
    assert_eq!((e1 - e2), Flags::A); // set difference
    assert_eq!(!e2, Flags::A); // set complement
    println!("e1={e1:#?}");
    let x = Flags::from_str("A | C").expect("Fail!");
    assert_eq!(e1, x);
}
