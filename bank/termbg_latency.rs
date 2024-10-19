/*[toml]
[dependencies]
crossterm = "0.28.1"
#termbg = "0.5.2"
termbg = { path = "/Users/donf/projects/termbg/" }
*/

/// Published example from `termbg` readme.
///
/// Detects the light or dark theme in use, as well as the colours in use.
//# Purpose: Demo theme detection with `termbg` and clearing terminal state with `crossterm`.
use std::time::Duration;
use termbg;

fn main() {
    println!("Latency={:?}", termbg::latency(Duration::from_secs(2)));
}
