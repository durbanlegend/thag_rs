/*[toml]
[dependencies]
clap = { version = "4.5.21", features = ["derive"] }
color-eyre = "0.6.3"
inquire = "0.7.5"
interactive-clap = "0.3.1"
shell-words = "1.1.0"
strum = { version = "0.26.3", features = ["derive"] }
strum_macros = "0.24"
*/

/// Published example from the `interactive-clap` crate. I've adapted the run instractions below for use with `thag_rs`:
///
/// This example shows additional functionality of the "interactive-clap" macro for parsing command-line data into a structure using macro attributes.
///
///```
/// thag demo/interactive_clap_adv_struct.rs (without parameters) => entered interactive mode
/// thag demo/interactive_clap_adv_struct.rs -- --age-full-years 30 --first-name QWE --second-name QWERTY --favorite-color red
///                                    => cli_args: CliArgs { age: Some(30), first_name: Some("QWE"), second_name: Some("QWERTY"), favorite_color: Some(Red) }
/// thag demo/interactive_clap_adv_struct.rs -- --first-name QWE --second-name QWERTY --favorite-color red
///                                    => cli_args: CliArgs { age: None, first_name: Some("QWE"), second_name: Some("QWERTY"), favorite_color: Some(Red) }
///```
///
/// To learn more about the parameters, use "help" flag:
///
///```
///  thag demo/interactive_clap_adv_struct.rs -- --help
///```
///
//# Purpose: Demo featured crate.
//# Categories: CLI, crates, technique
use clap;
use color_eyre;
use inquire::Select;
use interactive_clap::{ResultFromCli, ToCliArgs};
use shell_words;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
struct Args {
    #[interactive_clap(long = "age-full-years")]
    #[interactive_clap(skip_interactive_input)]
    /// If you want, enter the full age on the command line
    age: Option<u64>,
    #[interactive_clap(long)]
    /// What is your first name?
    first_name: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    second_name: String,
    #[interactive_clap(long)]
    #[interactive_clap(value_enum)]
    #[interactive_clap(skip_default_input_arg)]
    favorite_color: ColorPalette,
}

impl Args {
    fn input_second_name(_context: &()) -> color_eyre::eyre::Result<Option<String>> {
        match inquire::Text::new("Input second name".to_string().as_str()).prompt() {
            Ok(value) => Ok(Some(value)),
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn input_favorite_color(_context: &()) -> color_eyre::eyre::Result<Option<ColorPalette>> {
        let variants = ColorPaletteDiscriminants::iter().collect::<Vec<_>>();
        let selected = Select::new("What color is your favorite?", variants).prompt()?;
        match selected {
            ColorPaletteDiscriminants::Red => Ok(Some(ColorPalette::Red)),
            ColorPaletteDiscriminants::Orange => Ok(Some(ColorPalette::Orange)),
            ColorPaletteDiscriminants::Yellow => Ok(Some(ColorPalette::Yellow)),
            ColorPaletteDiscriminants::Green => Ok(Some(ColorPalette::Green)),
            ColorPaletteDiscriminants::Blue => Ok(Some(ColorPalette::Blue)),
            ColorPaletteDiscriminants::Indigo => Ok(Some(ColorPalette::Indigo)),
            ColorPaletteDiscriminants::Violet => Ok(Some(ColorPalette::Violet)),
        }
    }
}

#[derive(Debug, EnumDiscriminants, Clone, clap::ValueEnum)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum ColorPalette {
    #[strum_discriminants(strum(message = "red"))]
    /// Red
    Red,
    #[strum_discriminants(strum(message = "orange"))]
    /// Orange
    Orange,
    #[strum_discriminants(strum(message = "yellow"))]
    /// Yellow
    Yellow,
    #[strum_discriminants(strum(message = "green"))]
    /// Green
    Green,
    #[strum_discriminants(strum(message = "blue"))]
    /// Blue
    Blue,
    #[strum_discriminants(strum(message = "indigo"))]
    /// Indigo
    Indigo,
    #[strum_discriminants(strum(message = "violet"))]
    /// Violet
    Violet,
}

impl interactive_clap::ToCli for ColorPalette {
    type CliVariant = ColorPalette;
}

impl std::str::FromStr for ColorPalette {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "red" => Ok(Self::Red),
            "orange" => Ok(Self::Orange),
            "yellow" => Ok(Self::Yellow),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "indigo" => Ok(Self::Indigo),
            "violet" => Ok(Self::Violet),
            _ => Err("ColorPalette: incorrect value entered".to_string()),
        }
    }
}

impl std::fmt::Display for ColorPalette {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Red => write!(f, "red"),
            Self::Orange => write!(f, "orange"),
            Self::Yellow => write!(f, "yellow"),
            Self::Green => write!(f, "green"),
            Self::Blue => write!(f, "blue"),
            Self::Indigo => write!(f, "indigo"),
            Self::Violet => write!(f, "violet"),
        }
    }
}

impl std::fmt::Display for ColorPaletteDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Red => write!(f, "red"),
            Self::Orange => write!(f, "orange"),
            Self::Yellow => write!(f, "yellow"),
            Self::Green => write!(f, "green"),
            Self::Blue => write!(f, "blue"),
            Self::Indigo => write!(f, "indigo"),
            Self::Violet => write!(f, "violet"),
        }
    }
}

fn main() -> color_eyre::Result<()> {
    let mut cli_args = Args::parse();
    let context = (); // default: input_context = ()
    loop {
        let args = <Args as interactive_clap::FromCli>::from_cli(Some(cli_args), context);
        match args {
            ResultFromCli::Ok(cli_args) | ResultFromCli::Cancel(Some(cli_args)) => {
                println!("cli_args: {cli_args:?}");
                println!(
                    "Your console command:  {}",
                    shell_words::join(&cli_args.to_cli_args())
                );
                return Ok(());
            }
            ResultFromCli::Cancel(None) => {
                println!("Goodbye!");
                return Ok(());
            }
            ResultFromCli::Back => {
                cli_args = Default::default();
            }
            ResultFromCli::Err(cli_args, err) => {
                if let Some(cli_args) = cli_args {
                    println!(
                        "Your console command:  {}",
                        shell_words::join(&cli_args.to_cli_args())
                    );
                }
                return Err(err);
            }
        }
    }
}
