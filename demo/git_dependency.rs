/*[toml]
[dependencies]
rs-script = { git = "https://github.com/durbanlegend/rs-script" }
*/

/// Demo the use of git dependencies in the toml block. Local path dependencies
/// work the same way, e.g. `rs-script = { path = "/Users/donf/projects/rs-script" }`,
/// but obviously the path literal will be specific to your environment.
//# Purpose: Demo `git` dependencies, explain `path` dependencies.
use rs_script::colors;

fn main() {
    colors::main();
}
