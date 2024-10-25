#![allow(unused_macros)]

// Dummy implementation: rather than implementing operators, return string
// containing the symbols used, to make testing easy. The point of the exercise
// is that this macro should receive the generated set of symbols among its
// arguments (along with the type, the self/other identifiers and the
// implementation body). What it does with these is not important here, we
// simply have to make sure that it received the correct ones, so we return them
// as strings, which can be checked easily in a test.
#[macro_export]
macro_rules! doit {
    ($op:tt $opa:tt $T:ident $TA:ident $fn:ident $fna:ident for $Type:ty:
     ($self:ident, $other:ident) { $($body:tt)*}) => {
        format!(
            "{} {} {} {} {} {} for {}",
            stringify!($op),
            stringify!($opa),
            stringify!($T),
            stringify!($TA),
            stringify!($fn),
            stringify!($fna),
            stringify!($Type),
        );
    };
}

// Generate the 6 symbols that belong to any given operation and pass them to
// the core implementation (doit above) along with any other information passed
// by the user to the user-facing make_ops macro.
macro_rules! add_ops { ($($T:tt)*) => { doit!(+ += Add AddAssign add add_assign $($T)* ) } }
macro_rules! sub_ops { ($($T:tt)*) => { doit!(- -= Sub SubAssign sub sub_assign $($T)* ) } }
macro_rules! mul_ops { ($($T:tt)*) => { doit!(* *= Mul MulAssign mul mul_assign $($T)* ) } }
// And so on, for any other operators

// User-facing interface: Translate the operation's primary symbol (+, -, *,
// etc.) into the corresponding symbol generator (add_ops, sub_obs, etc.)
macro_rules! make_ops {
    (+ $($rest:tt)*) => { add_ops!($($rest)*) };
    (- $($rest:tt)*) => { sub_ops!($($rest)*) };
    (* $($rest:tt)*) => { mul_ops!($($rest)*) };
    // And so on, for any other operators
}

// Check that the implementation receives the correct symbols and type
#[test]
fn testit() {
    assert_eq! {
        make_ops!(+ for MyType: (self, other) { stuff }),
        "+ += Add AddAssign add add_assign for MyType"
    }

    assert_eq! {
        make_ops!(- for HisType: (self, other) { whatever }),
        "- -= Sub SubAssign sub sub_assign for HisType"
    }

    assert_eq! {
        make_ops!(* for Banana: (self, other) { peel }),
        "* *= Mul MulAssign mul mul_assign for Banana"
    }
}
