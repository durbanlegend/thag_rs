#![allow(dead_code)]
/// Exploring expansion
//# Purpose: explore proc macros
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveBasic;

#[derive(Debug, DeriveBasic)]
#[expand_macro]
struct MyStruct {
    field: String,
}

fn main() {
    let my_struct = MyStruct::new("Answer:".to_string());
    println!(
        "my_struct = {my_struct:#?}, my_struct.field={}",
        my_struct.field
    );
}
