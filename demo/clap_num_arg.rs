use clap::{Arg, Command};

/// `clap` with a numeric option.
///
/// E.g. `thag demo/clap_num_arg.rs -- 45`
//# Purpose: Basic demo of `clap` parsing a numeric argument
//# Categories: CLI, crates, technique
fn main() {
    let matches = Command::new("clap_num_arg")
        .arg(
            Arg::new("number")
                .help("The numeric value to process")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Extract the parsed u128 value
    let value: u128 = matches
        .get_one::<String>("number")
        .expect("Missing or invalid argument")
        .parse::<u128>()
        .unwrap();

    // Use the value as needed
    println!("Received value: {}", value);

    // Your code that utilizes the value can go here
}
