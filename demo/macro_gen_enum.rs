/// First prototype of building an enum from a macro and using it thereafter, thanks to SO user DK.
/// `https://stackoverflow.com/questions/37006835/building-an-enum-inside-a-macro`
//# Purpose: explore a technique for resolving mappings from a message level enum to corresponding
//# Categories: macros, technique
// message styles at compile time instead of dynamically while logging. This involves using macros
// to build impls for 4 enums representing the 4 combinations of light vs dark theme and 16 vs 256
// colour palette, and selecting the appropriate enum at the start of execution according to the
// user's choice of theme and the capabilities of the terminal. It all starts here!
macro_rules! build {
    ($($body:tt)*) => {
        as_item! {
            #[derive(Debug)]
            enum Test {
                $($body)*
            }
        }
    };
}
macro_rules! as_item {
    ($i:item) => {
        $i
    };
}
fn main() {
    build! {
        Foo, Bar
    }
    println!("Test::Bar={:?}", Test::Bar);
}
