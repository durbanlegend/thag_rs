/*[toml]
[dependencies]
# https://github.com/rust-lang/cargo/issues/8555
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["color_support", "core", "simplelog"] }
# thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["color_support", "core", "simplelog"] }
*/

use thag_rs::logging::V;
/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
//# Categories: technique
// The colors module was removed, so we'll just demonstrate a simple color message
use thag_rs::{cvprtln, Lvl};

// Simple color demonstration replacing the removed colors module
cvprtln!(
    Lvl::EMPHASIS,
    V::N,
    "This demonstrates git dependency usage with thag_rs"
);
cvprtln!(
    Lvl::NORMAL,
    V::N,
    "The colors module was removed after v0.1.9"
);
cvprtln!(
    Lvl::DEBUG,
    V::N,
    "Use demo/colors.rs for color demonstrations"
);
