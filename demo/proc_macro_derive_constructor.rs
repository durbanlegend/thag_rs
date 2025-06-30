#![allow(dead_code)]
/// Basic "derive" macro generates a constructor (`new()`) for the struct it annotates.
///
/// It also demonstrates how we can configure an attribute to expand the macro from the
/// caller.
//# Purpose: explore proc macros
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::DeriveConstructor;

#[derive(Debug, DeriveConstructor)]
#[expand_macro]
struct MyStruct {
    name: String,
    count: usize,
}

fn main() {
    let my_struct = MyStruct::new("Patrick:".to_string(), 35);
    println!(
        "my_struct = {my_struct:#?}, my_struct.name={}, my_struct.count={}",
        my_struct.name, my_struct.count
    );
}
