/*[toml]
[dependencies]
reedline-repl-rs = "1.1.1"
*/

/// Published example from `reedline-repl-rs` crate. This one uses the
/// `clap` builder pattern; there is also one using the`clap` derive pattern.
///
/// Example using Repl with Context
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
