/// Published example from `reedline-repl-rs` crate.
///
/// Sample invocation and dialogue:
///
/// ```bash
/// thag demo/reedline_repl.rs
/// Welcome to MyApp
/// MyApp〉say hello World!
/// Hello, World!
/// MyApp〉say goodbye --spanish                                                                                                                                06/30/2025 02:13:40 PM
/// Adiós!
/// MyApp〉[Ctrl-D]
/// $
/// ```
///
/// The latest version of this example is available in the [examples] folder in the `reedline-repl-rs` repository.
/// At time of writing you can run it successfully simply by invoking its URL with the `thag_url` tool
/// and passing the required arguments as normal, like this:
///
/// ```bash
/// thag_url https://github.com/arturh85/reedline-repl-rs/blob/main/examples/subcommands.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Explore the suitability of this crate for a Rust REPL. Conclusion: it's more geared to commands.
//# Categories: crates, REPL, technique
// Original `reedline-repl-rs` crate comments:
// Subcommands example
use reedline_repl_rs::clap::{Arg, ArgAction, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};

fn say<T>(args: ArgMatches, _context: &mut T) -> Result<Option<String>> {
    match args.subcommand() {
        Some(("hello", sub_matches)) => Ok(Some(format!(
            "Hello, {}",
            sub_matches.get_one::<String>("who").unwrap()
        ))),
        Some(("goodbye", sub_matches)) => Ok(Some(
            if sub_matches.get_flag("spanish") {
                "Adiós!"
            } else {
                "Goodbye!"
            }
            .to_string(),
        )),
        _ => panic!("Unknown subcommand {:?}", args.subcommand_name()),
    }
}

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .with_banner("Welcome to MyApp")
        .with_command(
            Command::new("say")
                .subcommand(
                    Command::new("hello")
                        .arg(Arg::new("who").required(true))
                        .arg(Arg::new("uppercase")),
                )
                .subcommand(
                    Command::new("goodbye").arg(
                        Arg::new("spanish")
                            .action(ArgAction::SetTrue)
                            .long("spanish"),
                    ),
                )
                .about("Greetings!"),
            say,
        );
    repl.run()
}
