/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/thag_proc_macros" }
*/
#![allow(dead_code)]
/// Experiment with proc macro to generate const at compile time.
//# Purpose: Exploration.
use thag_proc_macros::repeat_dash;

// const FLOWER_BOX_LEN: usize = 70; // Can't accept constant.
fn main() {
    repeat_dash!(70);
    println!("DASH_LINE={DASH_LINE:#?}");
}
