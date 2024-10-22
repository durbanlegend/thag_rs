/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

use thag_proc_macros::DeriveKeyMapList;

const MAPPINGS: [(i32, &str, &str); 7] = [
    (10, "Key bindings", "Description"),
    (20, "q, Esc", "Close the file dialog"),
    (30, "j, ↓", "Move down in the file list"),
    (40, "k, ↑", "Move up in the file list"),
    (50, "Enter", "Select the current item"),
    (60, "u", "Move one directory up"),
    (70, "I", "Toggle showing hidden files"),
];

#[derive(DeriveKeyMapList, Default)]
#[use_mappings(MAPPINGS)]
#[deluxe(
    delete = ["I", "u"],
    add = [
        (61, "u", "Up one"),
        (71, "I", "Toggle hidden")],
    )]
struct MyStruct;

fn main() {
    let my_struct: MyStruct = Default::default();

    my_struct.print_values(); // This will print the overridden values.
}
