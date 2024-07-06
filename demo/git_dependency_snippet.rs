/*[toml]
[dependencies]
rs-script = { git = "https://github.com/durbanlegend/rs-script" }
*/

/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
use rs_script::colors;

// The colors module has a "redundant" public main method for this demo.
colors::main();
