use inferno::flamegraph::{self, color::BasicPalette, Options, Palette};
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = File::open("thag-profile.folded")?;
    let reader = BufReader::new(input);
    let output = File::create("flamegraph.svg")?;

    let mut opts = Options::default();
    opts.title = "Thag Profile".to_string();
    opts.colors = Palette::Basic(BasicPalette::Aqua);

    // Configure time units
    opts.count_name = "μs".to_owned();
    opts.min_width = 0.1; // Show very small times

    // // Optional: Custom frame formatting
    // opts.frame_extra_text_formatter = Some(Box::new(|_, extra| {
    //     if let Some(time_str) = extra.strip_prefix('(') {
    //         format!(" {}", time_str)
    //     } else {
    //         String::new()
    //     }
    // }));

    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    flamegraph::from_lines(&mut opts, line_refs, output)?;
    println!("Flamegraph generated: flamegraph.svg");
    Ok(())
}
