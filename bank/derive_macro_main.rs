/*[toml]
[dependencies]
quote = "1.0.37"
syn = "2.0.79"
# macro_demo_derive = { path = "/Users/donf/projects/macro_demo/macro_demo_derive/" }
macro_demo_derive = { path = "../derive_macro_lib/" }
*/

//procedural_macro/src/main.rs
use macro_demo_derive::MyMacro;

pub trait MyTrait {
    fn hello();
}

#[derive(MyMacro)]
struct MyStruct;

fn main() {
    MyStruct::hello();
}
