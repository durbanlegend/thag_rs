#![allow(dead_code)]
/// Exploring integrated macro expansion, based on `demo/proc_macro_const_demo.rs`.
/// Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`.
//# Purpose: First working prototype of expanding proc macros for debugging purposes. See also `demo/proc_macro_const_demo_debug.rs`.
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_bank_proc_macros::const_demo_expand;

const TEST: &str = "Blah";
const ANSWER: i32 = 42;
fn test() -> i32 {
    ANSWER
}

#[derive(Debug)]
struct Test {
    value1: &'static str,
    value2: fn() -> i32,
}

const_demo_expand! {
    #[allow(clippy::approx_constant)]
    const F64_WITH_ATTRIB: f64 = 1.570796327;
    const MINUS_FLOAT: f64 = -1.0;

    const LARGE_USIZE: usize = 16_000_000;
    const MINUS_INT: i8 = -1;

    const INITIAL_VALUE: isize = 5;
    let value = math::new(INITIAL_VALUE);

    // 15
    const ADD: isize = value.add(10);
    // -5
    const SUB: isize = value.sub(20);
    // 25
    const SUB_SUB: isize = value.sub(-30);
    // 50
    const ADD_NEW_OBJECT: isize = value.add(math::new(25));

    let other_math = math::new(10);
    other_math.add(10);
    // 70
    const ADD_OBJECT: isize = value.add(other_math);

    let one = 1;
    let two = 2;
    let three = 3;

    const ARRAY: [usize; _] = [one, 2, three];

    const STR_ARRAY_SZ: usize = _;
    let temp = "A static string";
    const STR: [&str; STR_ARRAY_SZ] = [temp, "array"];

    let four_math = math::new(10);
    four_math.add_four(1, 3, 6, 9);
    const FOUR_MATH: i128 = four_math.get();

    const TEST_STRUCT: Test = expr::custom("Test {value1: TEST, value2: test}");
}

println!("ADD_OBJECT={ADD_OBJECT}");
println!("ARRAY={ARRAY:?}");
println!("STR={STR:?}");
println!("FOUR_MATH={FOUR_MATH:?}");
println!();
assert_eq!(TEST_STRUCT.value1, TEST);
assert_eq!((TEST_STRUCT.value2)(), ANSWER);

println!("TEST_STRUCT={TEST_STRUCT:#?}");
println!("(TEST_STRUCT.value2)())={:#?}", (TEST_STRUCT.value2)());
