#![allow(dead_code, unused_imports)]
/*[toml]
[dependencies]
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog"] }
*/
// Original code
use std::collections::HashMap;

struct Foo;
impl Foo {
    // Comment
    /// Doc comment
    fn bar(&self) -> u32 {
        1
    }

    // Already has profiling
    fn baz(&self) -> u32 {
        profile_method!("Foo::baz");
        2
    }
}

fn regular_function() {
    println!("hello");
}

// Already has profiling
fn profiled_function() {
    profile!("profiled_function");
    println!("already profiled");
}

fn main() {
    regular_function();
    profiled_function();
}
