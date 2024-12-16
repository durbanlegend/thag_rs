/*[toml]
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
*/


#![allow(dead_code)]
/// Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`
//# Purpose: Demo the use of proc macros to generate constants at compile time
use serde::{Deserialize, Serialize};
use serde_json::{self, to_string};

// "use thag_demo_proc_macros..." is a magic import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::const_demo_grail;

let v = vec![5, 6];
const_demo_grail! {
    let json_string = to_string(&v)?;

    const INITIAL_VALUE: Vec<i128> = json_string;
    // const UPD_VALUE = grail::new(INITIAL_VALUE);
    // const UPD_VALUE: isize = value::get();

}

println!("INITIAL_VALUE={INITIAL_VALUE}");
println!("UPD_VALUE={UPD_VALUE:?}");
