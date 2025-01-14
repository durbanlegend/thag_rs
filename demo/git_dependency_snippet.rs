/*[toml]
[dependencies]
thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "main" }
*/

/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
//# Categories: technique
use thag_rs::colors;

// The colors module has a "redundant" public main method for this demo.
colors::main();
