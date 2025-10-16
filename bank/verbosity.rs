/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs" }

ratatui = "=0.26.3"
# [patch.crates-io]
# ratatui = { version = "=0.26.3" }
*/

use thag_rs::log;
use thag_rs::Verbosity;

fn main() {
    log!(Verbosity::Quiet, "Quiet message");
    log!(Verbosity::Normal, "Normal message");
    log!(Verbosity::Verbose, "Verbose message");
}
