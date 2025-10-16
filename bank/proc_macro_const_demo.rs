#![allow(dead_code)]
/// Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`,
/// licensed under MIT and Apache-2 licences.
//# Purpose: Demo the use of proc macros to generate constants at compile time
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::const_demo;

struct Ary<'a> {
    x: &'a [&'static str],
}

pub fn get_ary<'a>(x: &'a [&'static str]) -> Ary<'a> {
    Ary { x }
}

fn main() {
    const_demo! {
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

        const STR2_ARRAY_SZ: usize = _;
        const STR2: [Ary; STR2_ARRAY_SZ] = [get_ary(&["A static", "string"]), get_ary(&["array"])];

    };

    // assert_eq!(VARIABLE, "A Variable");

    println!("ADD_OBJECT={ADD_OBJECT}");
    println!("ARRAY={ARRAY:?}");
    println!("STR={STR:?}");
    println!("FOUR_MATH={FOUR_MATH:?}");
    println!("STR2={STR2:?}");
}
