/*[toml]
[dependencies]
# quote = "1.0.37"
# syn = { version = "2.0.82", features = ["extra-traits", "full", "parsing", "printing"] }
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/src/proc_macros" }
*/

#![allow(dead_code, unused_variables)]
use thag_proc_macros::string_concat;

string_concat! {
    let first = "First";
    let second = "Second";
    const STRING: &str = string::concat(first, second);
}

assert_eq!(STRING, "FirstSecond");
println!("STRING={STRING}");
