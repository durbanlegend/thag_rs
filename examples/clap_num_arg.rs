use clap::{Arg, Command};

fn main() {
    let matches = Command::new("clap_num_arg")
        .arg(
            Arg::new("number")
                .help("The numeric value to process")
                .required(true)
                .index(1), // .value_name("VALUE")
        )
        .get_matches();

    // Extract the parsed u128 value
    let value: u128 = matches
        .get_one::<String>("number")
        .expect("REASON")
        .parse::<u128>()
        .unwrap();

    // Use the value as needed
    println!("Received value: {}", value);

    // Your code that utilizes the value can go here
}
