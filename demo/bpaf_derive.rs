/*[toml]
[dependencies]
bpaf = { version = "0.9.11", features = ["derive"] }
bpaf_derive = "0.5.10"
*/

/// Example from bpaf crate docs2/src/command/derive.rs.
///
/// E.g. `thag demo/bpaf_cmd_ex.rs -- --flag cmd --flag --arg=6`
//# Purpose: Demo CLI alternative to clap crate
//# Categories: CLI, crates, technique
use bpaf_derive::Bpaf;

#[derive(Debug, Clone, Bpaf)]
// `command` annotation with no name gets the name from the object it is attached to,
// but you can override it using something like #[bpaf(command("my_command"))]
// you can chain more short and long names here to serve as aliases
#[bpaf(command("cmd"), short('c'))]
// Command to do something
pub struct Cmd {
    /// This flag is specific to command
    flag: bool,
    arg: usize,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// This flag is specific to the outer layer
    flag: bool,
    #[bpaf(external)]
    cmd: Cmd,
}

fn main() {
    println!("{:?}", options().run())
}
