#![allow(clippy::uninlined_format_args)]
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "my_script", about = "A versatile script with various options.")]
struct CliOptions {
    #[structopt(subcommand)]
    command: Option<Command>,

    /// Activate verbose mode, printing additional information.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Show timings for each stage of script execution.
    #[structopt(short = "t", long = "timings")]
    timings: bool,

    /// Generate necessary files without building.
    #[structopt(short = "g", long = "generate")]
    generate: bool,

    /// Build the final project.
    #[structopt(short = "b", long = "build")]
    build: bool,

    /// Display help information.
    #[structopt(short = "h", long = "help")]
    help: bool,

    /// Display version information.
    #[structopt(short = "V", long = "version")]
    version: bool,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Run the actual script functionality.  You would add your script logic here.
    Run {
        #[structopt(name = "program_name")]
        program_name: Option<String>, // Make program_name optional
                                      // ... other options for Run
    },
    Help,
    Version,
}

fn main() {
    let options = CliOptions::from_args();

    match options.command {
        Some(Command::Run { program_name }) => {
            // Run command logic, handle Some(program_name) or None
            if let Some(name) = program_name {
                // Use the program name if provided
                println!("Running program: {}", name);
            } else {
                // Handle the case where no program name was provided
                println!("No program name specified. Use 'my_script run program_name'.");
            }
        }
        Some(Command::Help) => {
            CliOptions::clap().print_help().unwrap();
        }
        Some(Command::Version) => {
            println!("my_script version: 1.0.0"); // Update version as needed
        }
        None => {
            // Script was called without a subcommand, provide guidance or handle as needed
            println!("my_script: No subcommand specified. Use options like 'my_script --help' or 'my_script run program_name'.");
        }
    }

    // // Handle help and version information
    // if options.help {
    //     CliOptions::clap().print_help().unwrap();
    //     return;
    // }

    // if options.version {
    //     println!("my_script version: 1.0.0"); // Update version as needed
    //     return;
    // }

    // // Handle other options and subcommands based on your script's functionality
    // let mut script_args = Vec::new();
    // if let Some(command) = options.command {
    //     match command {
    //         Command::Run { program_name: x } => {
    //             // Add your script logic here
    //             println!("Running the script with options:");
    //             println!("Verbose: {}", options.verbose);
    //             println!("Timings: {}", options.timings);
    //             println!("Generate: {}", options.generate);
    //             println!("Build: {}", options.build);
    //         }
    //     }
    // } else {
    //     // Script was called without a subcommand, provide guidance or handle as needed
    //     println!("my_script: No subcommand specified. Use 'my_script run' or other subcommands.");
    // }
}
