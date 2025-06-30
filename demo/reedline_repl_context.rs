/*[toml]
[dependencies]
reedline-repl-rs = "1.1.1"
*/

/// Published example from `reedline-repl-rs` crate. This one uses the
/// `clap` builder pattern; there is also one using the`clap` derive pattern.
///
/// The latest version of this example is available in the [examples] folder in the `reedline-repl-rs` repository.
/// At time of writing you can run it successfully simply by invoking its URL with the `thag_url` tool
/// and passing the required arguments as normal, like this:
///
/// ```bash
/// thag_url https://github.com/arturh85/reedline-repl-rs/blob/main/examples/with_context.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Evaluation of featured crate and of using clap to structure command input.
//# Categories: crates, REPL, technique
// Original `reedline-repl-rs` crate comments:
// Example using Repl with Context
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::{Repl, Result};
use std::collections::VecDeque;

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

// Append name to list
fn append(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.get_one::<String>("name").unwrap().to_string();
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

// Prepend name to list
fn prepend(args: ArgMatches, context: &mut Context) -> Result<Option<String>> {
    let name: String = args.get_one::<String>("name").unwrap().to_string();
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(Context::default())
        .with_name("MyList")
        .with_version("v0.1.0")
        .with_description("My very cool List")
        .with_command(
            Command::new("append")
                .arg(Arg::new("name").required(true))
                .about("Append name to end of list"),
            append,
        )
        .with_command(
            Command::new("prepend")
                .arg(Arg::new("name").required(true))
                .about("Prepend name to front of list"),
            prepend,
        )
        .with_on_after_command(|context| Ok(Some(format!("MyList [{}]", context.list.len()))));
    repl.run()
}
