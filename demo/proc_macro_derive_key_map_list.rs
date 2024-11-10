/// Use a derive proc macro to implement a table. from a base with additions and deletions.
/// Not very useful currently: the dream is to generate a constant and get mappings as a variable.
//# Purpose: explore derive proc macros
// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveKeyMapList;

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
    let adjusted_mappings = my_struct.adjust_mappings();
    println!("adjusted_mappings={adjusted_mappings:#?}");
}
