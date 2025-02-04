#![allow(dead_code, unused_imports)]
/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog"] }
*/
// Original code
use std::collections::HashMap;

use thag_rs::{profile, profile_method};

struct Foo;
impl Foo {
    // Comment
    /// Doc comment
    fn bar(&self) -> u32 {
        profile_method!("Foo::bar");
        println!("In Foo::bar");
        1
    }

    // Already has profiling
    fn baz(&self) -> u32 {
        profile_method!("Foo::baz");
        println!("In Foo::baz");
        2
    }
}

fn regular_function(foo: &Foo) {
    profile!("regular_function");
    println!("hello");
    foo.bar();
}

// Already has profiling
fn profiled_function(foo: &Foo) {
    profile!("profiled_function");
    println!("already profiled");
    foo.baz();
}

fn main() {
    let _ = thag_rs::profiling::enable_profiling(true);
    let foo = Foo;
    regular_function(&foo);
    profiled_function(&foo);
}
