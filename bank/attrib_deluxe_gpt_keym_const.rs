/*[toml]
[dependencies]
#syn = { version = "2.0.85", features = ["extra-traits", "full"] }
syn = { version = "2.0.85", features = ["full"] }
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

#![allow(dead_code, unused_variables)]
// use syn::parse_quote;
// use syn::Expr;
// use syn::ItemConst;
use thag_proc_macros::use_mappings;

// Put this in tui_editor with #[macro_export]
macro_rules! base_data {
    () => {
        [
            (10, "Key bindings", "Description"),
            (20, "q, Esc", "Close the file dialog"),
            (30, "j, ↓", "Move down in the file list"),
            (40, "k, ↑", "Move up in the file list"),
            (50, "Enter", "Select the current item"),
            (60, "u", "Move one directory up"),
            (70, "I", "Toggle showing hidden files"),
        ]
    };
}

macro_rules! additions_data {
    () => {
        [(61, "u", "Up one"), (71, "I", "Toggle hidden")]
    };
}

macro_rules! deletions_data {
    () => {
        ["u", "I"]
    };
}

#[use_mappings(
    base = base_data!(),
    additions = additions_data!(),
    deletions = deletions_data!(),
)]
#[derive(Default)]
struct MyStruct;

// const MY_ADDITIONS_STRUCT: [(i32, &str, &str); 2] = [
//     (61, "u", "↑ Parent directory"),
//     (71, "I", "Show/hide special files"),
// ];

// const MY_DELETIONS_STRUCT: [&str; 2] = ["u", "I"];

#[macro_export]
macro_rules! to_tokens {
    (
        $(($seq:expr, $keys:expr, $desc:expr)),* $(,)?
    ) => {
        &[
            $(
                (
                    seq: $seq,
                    keys: $keys,
                    desc: $desc
                )
            ),*
        ]
    }
}

fn main() {
    // let my_struct: MyStruct = Default::default();

    // println!("MY_BASE_STRUCT={:?}", MY_BASE_STRUCT);

    // println!("MAPPINGS={:?}", MAPPINGS);

    // Print final mappings
    println!("FINAL_MAPPINGS={:#?}", FINAL_MAPPINGS);
    println!("ADDITIONS={:#?}", ADDITIONS);
}
