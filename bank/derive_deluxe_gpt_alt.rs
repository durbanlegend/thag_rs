/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

use thag_proc_macros::DeriveKeyMapList;

const MAPPINGS_1: [(i32, &str); 2] = [(1, "First"), (2, "Second")];
const MAPPINGS_2: [(i32, &str); 2] = [(3, "Third"), (4, "Fourth")];

#[derive(DeriveKeyMapList, Default)]
#[use_mappings(MAPPINGS_2)]
#[deluxe(
    delete = ["key2", "key4"],
    add = [
        (10, "key1", "desc1"),
        (20, "key2", "desc2"),
        (30, "key3", "desc3")],
    )]
struct MyStruct;

fn main() {
    let my_struct: MyStruct = Default::default();

    my_struct.print_values(); // This will print the overridden values.
}
