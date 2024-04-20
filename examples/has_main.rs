use regex::Regex;
use std::io::Write;

fn has_main(source: &str, verbose: bool) -> bool {
    let re = Regex::new(r"(?m)^\s*fn\s* main\(\s*\)").unwrap();
    let matches = re.find_iter(source).count();
    eprintln!("matches={matches}, verbose={verbose}");
    match matches {
        0 => {
            if verbose {
                println!("Source does not contain fn main(), thus a snippet");
            }
            false
        }
        1 => true,
        _ => {
            writeln!(
                &mut std::io::stderr(),
                "Invalid source, contains {matches} occurrences of fn main(), at most 1 is allowed"
            )
            .unwrap();
            std::process::exit(1);
        }
    }
}

fn main() {
    let source = r"fn
        main( )
          ";
    println!("has_main({source}) = {:#?}", has_main(source, true));
}
