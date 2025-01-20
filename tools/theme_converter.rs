/*
Usage:

Either compile the tool with `thag tools/theme_converter.rs -x` and run it as `theme_converter ...`
(recommended) or run it as `thag tools/theme_converter.rs -- ...`.

# Convert a single theme
thag tools/theme_converter.rs -- -i themes/base24/dracula.yaml -o themes/converted

# Convert a directory of themes
thag tools/theme_converter.rs -- -i themes/base24 -o themes/converted -v

# Convert and generate 256-color versions
thag tools/theme_converter.rs -- -i themes/base24 -o themes/converted -c -v

# Force overwrite existing themes
thag tools/theme_converter.rs -- -i themes/base24 -o themes/converted -f
*/

/*[toml]
[dependencies]
clap = { version = "4.5.26", features = ["cargo", "derive"] }
serde = { version = "1.0.217", features = ["derive"] }
serde_yaml_ok = "0.9.36"
# thag_rs = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", default-features = false, features = ["ast", "color_detect", "config", "simplelog"] }
thag_rs = { path = "/Users/donf/projects/thag_rs", default-features = false, features = ["ast", "color_detect", "config", "simplelog"] }
toml = "0.8.19"
*/

/// Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.
//# Purpose: Theme generation.
//# Categories: tools
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thag_rs::styling::{Palette, Style, Theme};
use thag_rs::{ColorSupport, TermBgLuma};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaseTheme {
    #[serde(alias = "name")]
    scheme: String,
    author: String,
    system: String,
    variant: String,
    #[serde(default)]
    description: Option<String>,
    palette: BasePalette,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BasePalette {
    base00: String, //Background
    base01: String,
    base02: String,
    base03: String,
    base04: String,
    base05: String,
    base06: String,
    base07: String,
    base08: String,
    base09: String,
    #[serde(alias = "base0A")]
    base0_a: String, // Yellow
    #[serde(alias = "base0B")]
    base0_b: String, // Green
    #[serde(alias = "base0C")]
    base0_c: String, // Cyan
    #[serde(alias = "base0D")]
    base0_d: String, // Blue
    #[serde(alias = "base0E")]
    base0_e: String, // Magenta
    #[serde(alias = "base0F")]
    base0_f: String, // Brown
    // Base24 additional colors
    #[serde(default)]
    base10: Option<String>,
    #[serde(default)]
    base11: Option<String>,
    base12: Option<String>,
    base13: Option<String>,
    base14: Option<String>,
    base15: Option<String>,
    base16: Option<String>,
    base17: Option<String>,
}

impl BaseTheme {
    fn is_base24(&self) -> bool {
        self.palette.base10.is_some()
    }

    fn convert_to_thag(&self) -> Result<Theme, Box<dyn std::error::Error>> {
        let palette = if self.is_base24() {
            self.create_base24_palette()?
        } else {
            self.create_base16_palette()?
        };

        Ok(Theme {
            name: self.scheme.clone(),
            description: self
                .description
                .clone()
                .unwrap_or_else(|| format!("Converted from {} theme", self.scheme)),
            term_bg_luma: detect_background_luma(&self.palette.base00)?,
            min_color_support: ColorSupport::TrueColor,
            palette,
            background: Some(format!("{}", self.palette.base00)),
            is_builtin: false,
            filename: PathBuf::new(), // Will be set by caller
        })
    }

    fn create_base24_palette(&self) -> Result<Palette, Box<dyn std::error::Error>> {
        Ok(Palette {
            heading1: Style::from_fg_hex(&self.palette.base08)?.bold(), // Red
            heading2: Style::from_fg_hex(&self.palette.base0_d)?.bold(), // Blue
            heading3: Style::from_fg_hex(&self.palette.base0_c)?.bold(), // Cyan
            error: Style::from_fg_hex(&self.palette.base08)?,           // Red
            warning: Style::from_fg_hex(&self.palette.base09)?,         // Orange
            success: Style::from_fg_hex(&self.palette.base0_b)?,        // Green
            info: Style::from_fg_hex(&self.palette.base0_d)?,           // Blue
            emphasis: Style::from_fg_hex(&self.palette.base0_e)?,       // Magenta
            code: Style::from_fg_hex(&self.palette.base0_b)?,           // Green
            normal: Style::from_fg_hex(&self.palette.base05)?,          // Default foreground
            subtle: Style::from_fg_hex(self.palette.base14.as_ref().unwrap())?, // Light grey
            hint: Style::from_fg_hex(self.palette.base13.as_ref().unwrap())?.italic(),
            debug: Style::from_fg_hex(self.palette.base12.as_ref().unwrap())?,
            trace: Style::from_fg_hex(self.palette.base11.as_ref().unwrap())?,
        })
    }

    fn create_base16_palette(&self) -> Result<Palette, Box<dyn std::error::Error>> {
        Ok(Palette {
            heading1: Style::from_fg_hex(&self.palette.base08)?.bold(), // Red
            heading2: Style::from_fg_hex(&self.palette.base0_d)?.bold(), // Blue
            heading3: Style::from_fg_hex(&self.palette.base0_c)?.bold(), // Cyan
            error: Style::from_fg_hex(&self.palette.base08)?,           // Red
            warning: Style::from_fg_hex(&self.palette.base0_a)?,        // Yellow
            success: Style::from_fg_hex(&self.palette.base0_b)?,        // Green
            info: Style::from_fg_hex(&self.palette.base0_d)?,           // Blue
            emphasis: Style::from_fg_hex(&self.palette.base0_e)?,       // Magenta
            code: Style::from_fg_hex(&self.palette.base0_b)?,           // Green
            normal: Style::from_fg_hex(&self.palette.base05)?,          // Default foreground
            subtle: Style::from_fg_hex(&self.palette.base03)?,          // Comments color
            hint: Style::from_fg_hex(&self.palette.base03)?.italic(),
            debug: Style::from_fg_hex(&self.palette.base02)?,
            trace: Style::from_fg_hex(&self.palette.base01)?,
        })
    }
}

#[derive(Parser)]
#[command(author, version, about = "Convert Base16/24 themes to thag format")]
struct Cli {
    /// Input theme file or directory
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for converted themes
    #[arg(short, long, default_value = "themes/converted")]
    output: PathBuf,

    /// Force overwrite existing files
    #[arg(short, long)]
    force: bool,

    /// Generate 256-color versions
    #[arg(short = 'c', long)]
    color256: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color length".into());
    }
    Ok((
        u8::from_str_radix(&hex[0..2], 16)?,
        u8::from_str_radix(&hex[2..4], 16)?,
        u8::from_str_radix(&hex[4..6], 16)?,
    ))
}

fn detect_background_luma(hex: &str) -> Result<TermBgLuma, Box<dyn std::error::Error>> {
    let (r, g, b) = hex_to_rgb(hex)?;
    let luma = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
    Ok(if luma > 0.5 {
        TermBgLuma::Light
    } else {
        TermBgLuma::Dark
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&cli.output)?;

    if cli.input.is_dir() {
        convert_directory(&cli)?;
    } else {
        convert_file(&cli.input, &cli)?;
    }

    Ok(())
}

fn convert_directory(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(&cli.input)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if cli.verbose {
                println!("Converting {:?}", path);
            }
            convert_file(&path, cli)?;
        }
    }
    Ok(())
}

fn convert_file(input: &Path, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse YAML
    let content = fs::read_to_string(input)?;
    let theme: BaseTheme = serde_yaml_ok::from_str(&content)?;

    // Create output filename
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid input filename")?;

    // Convert to thag theme
    let thag_theme = theme.convert_to_thag()?;

    // Generate TOML
    let true_color_path = cli.output.join(format!("{}.toml", stem));
    if !cli.force && true_color_path.exists() {
        eprintln!("Skipping existing file: {:?}", true_color_path);
    } else {
        let toml = toml::to_string_pretty(&thag_theme.to_output())?;
        fs::write(&true_color_path, toml)?;
        if cli.verbose {
            println!("Created {:?}", true_color_path);
        }
    }

    // Optionally generate 256-color version
    if cli.color256 {
        let color256_path = cli.output.join(format!("{}_256.toml", stem));
        if !cli.force && color256_path.exists() {
            eprintln!("Skipping existing file: {:?}", color256_path);
        } else {
            let theme_256 = thag_theme.to_256_color()?;
            // let toml = toml::to_string_pretty(&theme_256)?;
            let toml = toml::to_string_pretty(&theme_256.to_output())?;
            fs::write(&color256_path, toml)?;
            if cli.verbose {
                println!("Created {:?}", color256_path);
            }
        }
    }

    Ok(())
}
