/*[toml]
[dependencies]
time = "0.1.25"
*/

use time::now;

/// Uses an old version of the time crate so we need the toml block
fn main() {
    println!("{}", now().rfc822z());
}
