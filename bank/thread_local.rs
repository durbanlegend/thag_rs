/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["core", "simplelog"] }
*/

use std::cell::Cell;
use std::sync::Arc;
use std::thread;
use thread_local::ThreadLocal;

use thag_rs::{enable_profiling, end_profile_section, profile_section, profiling};

#[enable_profiling]
fn main() {
    profile_section!("new");
    let tls = Arc::new(ThreadLocal::new());
    let _ = end_profile_section("new");

    // Create a bunch of threads to do stuff
    profile_section!("spawn");
    for _ in 0..5 {
        let tls2 = tls.clone();
        thread::spawn(move || {
            profile_section!("thread");
            // Increment a counter to count some event...
            let cell = tls2.get_or(|| Cell::new(0));
            cell.set(cell.get() + 1);
        })
        .join()
        .unwrap();
    }
    let _ = end_profile_section("spawn");

    // Once all threads are done, collect the counter values and return the
    // sum of all thread-local counter values.
    profile_section!("sum");
    let tls = Arc::try_unwrap(tls).unwrap();
    let total = tls.into_iter().fold(0, |x, y| x + y.get());
    assert_eq!(total, 5);
}
