/*[toml]
[dependencies]
env_logger = { version = "0.11.8", optional = true }
log = "0.4"
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "env_logger"] }

[features]
simplelog = []
default=["env_logger"]
debug-logs = []
env_logger = ["dep:env_logger"]
*/
use env_logger;
use env_logger::{Env,Builder};
use log::{info, debug};
use thag_rs::{debug_log, V, set_global_verbosity};
use thag_rs::logging::{enable_debug_logging, configure_log};

set_global_verbosity(V::V)?;
configure_log();
enable_debug_logging();

// let env = Env::new().filter("RUST_LOG");
// Builder::new().parse_env(env).init();
// info!("Initialized env_logger");
info!("Hello world!");
debug!("Woohoo!");
debug_log!("Vox clamantis in deserto!");
