/*[toml]
[dependencies]
clap = { version = "4.5.13", features = ["cargo", "derive"] }
*/

/// Published example from `clap` tutorial (derive), with added displays.
///
/// E.g. thag_rs demo/clap_tut_derive_03_04_subcommands.rs -- add spongebob
//# Purpose: Demonstrate `clap` CLI using the derive option
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Add { name: Option<String> },
}

fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Add { name } => {
            println!("'myapp add' was used, name is: {name:?}");
        }
    }
}
