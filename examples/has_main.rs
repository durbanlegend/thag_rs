use regex::Regex;
use std::error::Error;
use std::io::Write;

fn has_main(source: &str, verbose: bool) -> Result<bool, Box<dyn Error>> {
    // let re = Regex::new(r"(\bfn\s+main\(\)\s*\{)").unwrap(); //Gemini
    let re = Regex::new(r"(?x)\bfn\s* main\(\s*\)\s*\{").unwrap();
    let matches = re.find_iter(source).count();
    eprintln!("matches={matches}, verbose={verbose}");
    match matches {
        0 => {
            if verbose {
                println!("source does not contain fn main(), thus a snippet");
            }
            Ok(false)
        }
        1 => Ok(true),
        _ => {
            writeln!(&mut std::io::stderr(), "{}", "Invalid source, contains {matches} occurrences of fn main(), at most 1 is allowed").unwrap();
            std::process::exit(1);
        }
    }
}

fn main( )
{
    let source = r"fn
        main( )
          ";
    println!("has_main({source}) = {:#?}", has_main(source, true));
}
