/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

use thag_proc_macros::DeserializeVec;

#[derive(DeserializeVec, Default)]
#[deluxe(items = [(1, "First"), (2, "Second")])]
struct MyStruct;

fn main() {
    let example: MyStruct = Default::default();
    example.print_values();
}