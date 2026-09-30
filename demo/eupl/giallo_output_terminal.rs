/*[toml]
[dependencies]
giallo = {version = "^0.5.2", features = [ "dump" ] }
*/
// SPDX-License-Identifier: EUPL-1.2
//
// This specific demo file is licensed under the EUPL-1.2 because it
// demonstrates integration with the `giallo` library.
//
/// Highlight a Rust source with a giallo theme, printing the highlighted output to the terminal.
/// Published example from the `giallo` crate.
///
/// E.g.: `thag demo/giallo_output_terminal.rs -- demo/hello.rs rust catppuccin-frappe
//# Purpose: demo and test `egui_extras` code block formatting
//# Categories: crates, demo, styling, technique
//# Argument: PATH: Path to a `syntect` `.tmTheme` file. There are a few examples in the `thag_rs/assets/sublime_themes` directory and `egui_extras` has its own.
use std::env;
use std::fs;

use giallo::{HighlightOptions, Registry, ThemeVariant};
use giallo::{RenderOptions, TerminalRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <file_path> <language> <theme> [dark_theme]",
            args[0]
        );
        eprintln!("Examples:");
        eprintln!(
            "  Single theme:     cargo run --example output_terminal --features dump -- file.js javascript catppuccin-frappe"
        );
        std::process::exit(1);
    }

    let file_path = &args[1];
    let language = &args[2];
    let theme = &args[3];

    let mut registry = Registry::builtin()?;
    registry.link_grammars();

    let file_content = fs::read_to_string(file_path)?;

    let options = HighlightOptions::new(language, ThemeVariant::Single(theme));

    let highlighted = registry.highlight(&file_content, &options)?;
    let render_options = RenderOptions {
        show_line_numbers: true,
        ..Default::default()
    };
    let rendered = TerminalRenderer::default().render(&highlighted, &render_options);

    println!("{rendered}");

    Ok(())
}
