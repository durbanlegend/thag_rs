/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
log = "0.4"

[features]
default=["debug-logs"]
debug-logs = []
simplelog = []
*/
use thag_rs::debug_log;
use thag_rs::logging::{enable_debug_logging, configure_log};

configure_log();
enable_debug_logging();
debug_log!("Hello world!");
