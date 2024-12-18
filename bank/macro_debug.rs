/*[toml]
[dependencies]
crokey = "1.1.0"
*/

use crokey::KeyCombination;
use crokey::*;

#[macro_export]
macro_rules! key {
    ($($tt:tt)*) => {
        crokey::__private::key!(($crate) $($tt)*)
        // println!(($crate) $($tt)*)
    };
}

fn main() {
    println!("key!(ctrl - q) = {:?}", key!(ctrl - q));
}
