/*[toml]
[dependencies]
#once_cell = "1.20.2"
phf = { version = "0.11.3", features = ["macros"] }
toml = "0.8.19"
*/

/// Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
/// `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.
//# Purpose: Sample model of a basic function-like proc macro.
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
// use phf;
// use once_cell;
// use std::collections::HashMap;
use thag_demo_proc_macros::load_static_map;

#[derive(Debug, Clone)]
pub enum Value {
    String(&'static str),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(&'static [Value]),
    Table(&'static [(&'static str, Value)]),
    Datetime(&'static str),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Table(entries) => entries.iter().find(|(k, _)| *k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    // Optionally, add convenience methods for common types:
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&'static [Value]> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

fn main() {
    // This must be a string literal representing a path relative to the project dir,
    // assumed to be 2 steps obove the demo proc macro dir.
    load_static_map!("themes/built_in/dracula.toml");

    println!("description={:?}", MAP.get("description"));
    println!();
    println!("palette={:?}", MAP.get("palette"));
    println!();

    if let Some(value) = MAP.get("palette") {
        if let Some(nested_value) = value.get("success") {
            match nested_value {
                Value::String(s) => println!("Got string: {}", s),
                Value::Integer(i) => println!("Got integer: {}", i),
                // ... handle other variants
                _ => println!("Got  {nested_value:?}"),
            }
        }
    }
    println!();

    // // Or using the convenience methods:
    // if let Some(value) = MAP.get("some_table") {
    //     if let Some(nested_value) = value.get("nested_key") {
    //         if let Some(s) = nested_value.as_str() {
    //             println!("Got string: {}", s);
    //         }
    //     }
    // }

    // // You could even chain them:
    // let nested_str = MAP.get("some_table")
    //     .and_then(|v| v.get("nested_key"))
    //     .and_then(|v| v.as_str());

    // let palette = MAP.get("palette").unwrap();
    // println!("palette={:?}", palette);
    // println!("success={:?}", palette.get("success"));
    // println!(
    //     "palette.success={:#?}",
    //     MAP.get("palette").unwrap().get("success")
    // );
}
