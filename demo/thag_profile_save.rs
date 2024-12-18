/*[toml]
[dependencies]
inferno = "0.12.0"
inquire = "0.7.5"
*/

use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use inquire::MultiSelect;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First read all lines and collect unique function names
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // Extract unique function names
    let functions: HashSet<String> = lines
        .iter()
        .map(|line| {
            line.split(';')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        })
        .collect();
    let mut function_list: Vec<String> = functions.into_iter().collect();
    function_list.sort(); // Sort alphabetically

    // Let user select functions to filter out
    let to_filter = MultiSelect::new("Select functions to filter out:", function_list).prompt()?;

    // Filter the lines
    let filtered_lines: Vec<&str> = lines
        .iter()
        .filter(|line| {
            let func_name = line
                .split(';')
                .next()
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("");
            !to_filter.iter().any(|f| f == func_name)
        })
        .map(|s| s.as_str())
        .collect();

    // Generate flamegraph with filtered data
    let output = File::create("flamegraph.svg")?;
    let mut opts = Options::default();
    opts.title = "Thag Profile (Filtered)".to_string();
    opts.colors = Palette::Basic(BasicPalette::Aqua);
    opts.count_name = "μs".to_owned();
    opts.min_width = 0.1;

    flamegraph::from_lines(&mut opts, filtered_lines, output)?;
    println!("Filtered flamegraph generated: flamegraph.svg");
    Ok(())
}
