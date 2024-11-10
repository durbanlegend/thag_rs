/*[toml]
[dependencies]
thag_demo_proc_macros = { path = "/Users/donf/projects/thag_rs/demo/proc_macros" }
*/

#![allow(dead_code)]
use thag_demo_proc_macros::DeserializeVec;

const MAPPINGS: [(i32, &str, &str); 2] = [(1, "First"), (2, "Second")];
const MAPPINGS_2: [(i32, &str, &str); 2] = [(3, "Third"), (4, "Fourth")];

#[derive(DeserializeVec, Default)]
#[deluxe(items = [(9, "Ninth"), (10, "Tenth")])]
#[use_mappings(MAPPINGS_1)]
struct MyStruct {
    items: Vec<(i32, String)>,
}

fn main() {
    let example: MyStruct = Default::default();
    example.print_values();
}
