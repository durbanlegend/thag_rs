/*[toml]
[dependencies]
# rev may be necessary to force cargo to update to the latest commit, see
# https://github.com/rust-lang/cargo/issues/8555
rs-script = { git = "https://github.com/durbanlegend/rs-script", rev = "80dbcd8a03cc0dee98026fad67120ca59254ffc2"}
*/

/// Demo the use of git dependencies in the toml block. Local path dependencies
/// work the same way, e.g. `rs-script = { path = "/Users/donf/projects/rs-script" }`,
/// but obviously the path literal will be specific to your environment.
//# Purpose: Demo `git` dependencies, explain `path` dependencies.
use rs_script::colors;

fn main() {
    colors::main();
}