/*[toml]
[dependencies]
prettyplease = "0.2"
syn = { version = "2", default-features = false, features = ["full", "parsing"] }
*/

/// Published example from `prettyplease` Readme.
//# Purpose: Demo featured crate.
const INPUT: &str = stringify! {
    use crate::{
          lazy::{Lazy, SyncLazy, SyncOnceCell}, panic,
        sync::{ atomic::{AtomicUsize, Ordering::SeqCst},
            mpsc::channel, Mutex, },
      thread,
    };
    impl<T, U> Into<U> for T where U: From<T> {
        fn into(self) -> U { U::from(self) }
    }
};

fn main() {
    let syntax_tree = syn::parse_file(INPUT).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    print!("{}", formatted);
}
