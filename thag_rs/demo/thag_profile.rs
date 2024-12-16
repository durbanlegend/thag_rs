use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the folded stack trace
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);

    // Create output file
    let output = File::create("flamegraph.svg")?;

    // Configure flamegraph options
    let mut opts = Options::default();
    opts.title = "Thag Profile".to_string();
    opts.colors = Palette::Basic(BasicPalette::Hot);

    // Generate the flamegraph
    // Convert String to &str before passing to from_lines
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    flamegraph::from_lines(&mut opts, line_refs, output)?;

    println!("Flamegraph generated: flamegraph.svg");
    Ok(())
}
