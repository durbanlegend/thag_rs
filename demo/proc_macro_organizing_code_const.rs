#![allow(dead_code, unused_variables, clippy::redundant_pub_crate)]
/// Experimental - work in progress
//# Purpose: investigate the possibility of generating a useful constant.
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveConst;

const MAPPINGS: [(i32, &str, &str); 7] = [
    (10, "Key bindings", "Description"),
    (20, "q, Esc", "Close the file dialog"),
    (30, "j, ↓", "Move down in the file list"),
    (40, "k, ↑", "Move up in the file list"),
    (50, "Enter", "Select the current item"),
    (60, "u", "Move one directory up"),
    (70, "I", "Toggle showing hidden files"),
];

// // Put this in tui_editor with #[macro_export]
// macro_rules! base_data {
//     () => {
//         const MAPPINGS: [(i32, &str, &str); 7] = [
//             (10, "Key bindings", "Description"),
//             (20, "q, Esc", "Close the file dialog"),
//             (30, "j, ↓", "Move down in the file list"),
//             (40, "k, ↑", "Move up in the file list"),
//             (50, "Enter", "Select the current item"),
//             (60, "u", "Move one directory up"),
//             (70, "I", "Toggle showing hidden files"),
//         ];
//     };
// }

// macro_rules! additions_data {
//     () => {
//         [(61, "u", "Up one"), (71, "I", "Toggle hidden")]
//     };
// }

// #[use_mappings(
//     base = base_data!(),
//     additions = additions_data!(),
// )]
#[derive(Default, DeriveConst)]
// #[use_mappings(base_data!())]
#[use_mappings(MAPPINGS)]
#[adjust(
    delete = ["I", "u"],
    add = [
        (61, "u", "Up one"),
        (71, "I", "Toggle hidden")],
    )]
struct MyStruct;

fn main() {
    println!("Hello world!");
}
