/*[toml]
[dependencies]
reedline-repl-rs = "1.1.1"
*/

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
