/*[toml]
[dependencies]
# rev may be necessary to force cargo to update to the latest commit, see
# https://github.com/rust-lang/cargo/issues/8555
thag_rs = { git = "https://github.com/durbanlegend/thag_rs" }
*/

/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
use thag_rs::colors;

// The colors module has a "redundant" public main method for this demo.
colors::main();
