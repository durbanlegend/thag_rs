/*[toml]
[dependencies]
 bpaf = { version = "0.9.11", features = ["derive"] }
 bpaf_derive = "0.5.10"
*/

//# Purpose: Demo CLI alternative to clap crate
//# Crates: `bpaf`, `bpaf_derive`
//# Target: all
//# Type: program

/// Example from bpaf crate docs2/src/adjacent_command/derive.rs.
///
/// E.g. `rs_script demo/bpaf_cmd-chain.rs -- eat Fastfood drink --coffee sleep --time=5`
use bpaf_derive::Bpaf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(short, long)]
    /// Opt in for premium serivces
    pub premium: bool,
    #[bpaf(external(cmd), many)]
    pub commands: Vec<Cmd>,
}

#[derive(Debug, Clone, Bpaf)]
pub enum Cmd {
    #[bpaf(command, adjacent)]
    /// Performs eating action
    Eat(#[bpaf(positional("FOOD"))] String),
    #[bpaf(command, adjacent)]
    /// Performs drinking action
    Drink {
        /// Are you going to drink coffee?
        coffee: bool,
    },
    #[bpaf(command, adjacent)]
    /// Performs taking a nap action
    Sleep {
        #[bpaf(argument("HOURS"))]
        time: usize,
    },
}

fn main() {
    println!("{:?}", options().run())
}
