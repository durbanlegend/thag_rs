extern crate clap;

use clap::{App, Arg};

// fn xmain() {
    let matches = App::new("Script Runner")
        .version("1.0")
        .author("Your Name")
        .about("Runs scripts with arguments and flags")
        .arg(
            Arg::with_name("script")
                .help("Sets the script to run")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("args")
                .help("Sets the arguments for the script")
                .multiple(true)
                .last(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("timings")
                .short("t")
                .long("timings")
                .takes_value(false)
                .help("Displays timings"),
        )
        .arg(
            Arg::with_name("generate")
                .short("g")
                .long("generate")
                .takes_value(false)
                .help("Generates something"),
        )
        .arg(
            Arg::with_name("build")
                .short("b")
                .long("build")
                .takes_value(false)
                .help("Builds something"),
        )
        .get_matches();

    if matches.is_present("verbose") {
        println!("Verbosity enabled");
    }

    if matches.is_present("timings") {
        println!("Timings enabled");
    }

    if matches.is_present("generate") {
        println!("Generating something");
    }

    if matches.is_present("build") {
        println!("Building something");
    }

    match matches.value_of("script") {
        Some(script) => {
            println!("Running script: {}", script);
            if let Some(args) = matches.values_of("args") {
                println!("With arguments:");
                for arg in args {
                    println!("{}", arg);
                }
            }
        }
        None => println!("No script provided"),
    }
// }
