#![allow(clippy::uninlined_format_args)]
use structopt::StructOpt;
use structopt::clap::AppSettings;
// use structopt::validator::ValidationError;


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

    /// Display help information (short option).
      #[structopt(short = "h", long = "help")]
      help: bool,

      /// Display version information (short option).
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
    // Help,
    // Version,
}

// fn validate_no_program_if_help_or_version(options: &CliOptions) -> Result<(), ValidationError> {
//   if (options.help || options.version) && options.command.program_name.is_empty() {
//     // --help or --version specified, program name is allowed to be missing
//     Ok(())
//   } else if options.program_name.is_empty() && !options.help && !options.version {
//     Err(ValidationError::new("Program name required. Use 'my_script run program_name'."))
//   } else {
//     Ok(())
//   }
// }

eprintln!("Here!");

let options = CliOptions::from_args().command.expect("Error parsing arguments");
println!("options={:?}", options);
  // let validation_result = validate_no_program_if_help_or_version(&options);

  // if let Err(err) = validation_result {
  //     eprintln!("Error: {}", err);
  //     return Ok(());
  // }

eprintln!("Still here!");
// println!("options.command={:?}", options.command);

// match options.command {
//     Some(Command::Run { program_name }) => {
//       // Run command logic...
//       println!("Run command logic here");
//     }
//     Some(CliOptions::help) => {
//       CliOptions::clap().print_help().unwrap();
//       return Ok(());
//     }
//     Some(CliOptions::version) => {
//       println!("my_script version: 1.0.0"); // Update version as needed
//       return Ok(());
//     }
//     None => {
//       // Script was called without a subcommand, provide guidance or handle as needed
//       println!("my_script: No subcommand specified. Use options like 'my_script --help' or 'my_script run program_name'.");
//     }
//   }
