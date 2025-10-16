/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.
///
/// ## Usage examples:
///
/// ### Convert a single theme
///
/// ```Rust
/// thag_convert_themes -i themes/wezterm/atelier_seaside_light.yaml -o themes/converted
/// ```
///
/// ### Convert a directory of themes (verbosely)
///
/// ```Rust
/// thag_convert_themes -i themes/wezterm -o themes/converted -v
/// ```
///
/// ### Convert and also generate 256-color versions (verbosely)
///
/// ```Rust
/// thag_convert_themes -i themes/wezterm -o themes/converted -c -v
/// ```
///
/// ### Force overwrite existing themes
///
/// ```Rust
/// thag_convert_themes -i themes/wezterm -o themes/converted -f
/// ```
///
//# Purpose: Theme generation.
//# Categories: tools
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thag_styling::{
    auto_help, find_closest_color, hsl_to_rgb, rgb_to_hsl, ColorSupport, ColorValue, Palette,
    Style, TermBgLuma, Theme,
};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaseTheme {
    #[serde(alias = "name")]
    scheme: String,
    author: String,
    system: Option<String>,
    variant: Option<String>,
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
    const fn is_base24(&self) -> bool {
        self.palette.base10.is_some()
    }

    /// Extract base16/24 colors as RGB array for ANSI terminal mapping
    fn extract_base_colors(&self) -> Result<Vec<[u8; 3]>, Box<dyn std::error::Error>> {
        let to_array = |hex: &str| -> Result<[u8; 3], Box<dyn std::error::Error>> {
            let rgb = hex_to_rgb(hex)?;
            Ok(rgb)
        };

        let mut colors = vec![
            to_array(&self.palette.base00)?,
            to_array(&self.palette.base01)?,
            to_array(&self.palette.base02)?,
            to_array(&self.palette.base03)?,
            to_array(&self.palette.base04)?,
            to_array(&self.palette.base05)?,
            to_array(&self.palette.base06)?,
            to_array(&self.palette.base07)?,
            to_array(&self.palette.base08)?,
            to_array(&self.palette.base09)?,
            to_array(&self.palette.base0_a)?,
            to_array(&self.palette.base0_b)?,
            to_array(&self.palette.base0_c)?,
            to_array(&self.palette.base0_d)?,
            to_array(&self.palette.base0_e)?,
            to_array(&self.palette.base0_f)?,
        ];

        // Add base24-specific colors if present
        if self.is_base24() {
            if let Some(ref base10) = self.palette.base10 {
                colors.push(to_array(base10)?);
            }
            if let Some(ref base11) = self.palette.base11 {
                colors.push(to_array(base11)?);
            }
            if let Some(ref base12) = self.palette.base12 {
                colors.push(to_array(base12)?);
            }
            if let Some(ref base13) = self.palette.base13 {
                colors.push(to_array(base13)?);
            }
            if let Some(ref base14) = self.palette.base14 {
                colors.push(to_array(base14)?);
            }
            if let Some(ref base15) = self.palette.base15 {
                colors.push(to_array(base15)?);
            }
            if let Some(ref base16) = self.palette.base16 {
                colors.push(to_array(base16)?);
            }
            if let Some(ref base17) = self.palette.base17 {
                colors.push(to_array(base17)?);
            }
        }

        Ok(colors)
    }

    fn convert_to_thag(&self) -> Result<Theme, Box<dyn std::error::Error>> {
        let palette = if self.is_base24() {
            self.create_base24_palette()?
        } else {
            self.create_base16_palette()?
        };

        let bg = self.palette.base00.trim_start_matches('#').to_lowercase();
        let backgrounds = vec![format!("#{bg}")];
        let bg_rgbs = vec![hex_to_rgb(&bg)?];

        let is_light_theme = matches!(
            detect_background_luma(&self.palette.base00)?,
            TermBgLuma::Light
        );
        let bg_rgb = bg_rgbs[0];
        let enhanced_palette = Self::enhance_palette_contrast(palette, bg_rgb, is_light_theme);

        Ok(Theme {
            name: self.scheme.clone(),
            description: self
                .description
                .clone()
                .unwrap_or_else(|| format!("Converted from {} theme", self.scheme)),
            term_bg_luma: detect_background_luma(&self.palette.base00)?,
            min_color_support: ColorSupport::TrueColor,
            palette: enhanced_palette,
            backgrounds,
            bg_rgbs,
            is_builtin: false,
            filename: PathBuf::new(), // Will be set by caller
            base_colors: Some(self.extract_base_colors()?),
        })
    }

    fn create_base24_palette(&self) -> Result<Palette, Box<dyn std::error::Error>> {
        // Perfect 1:1 mapping: Base16 colors (base00-base0F) to thag roles
        Ok(Palette {
            // base00 -> Background (not a role, handled separately)
            // base01 -> Subtle (darker background/line highlighting)
            subtle: Style::from_fg_hex(&self.palette.base01)?,
            // base02 -> Commentary (selection background/comments)
            commentary: Style::from_fg_hex(&self.palette.base02)?,
            // base03 -> Hint (comments/invisibles)
            hint: Style::from_fg_hex(&self.palette.base03)?.italic(),
            // base04 -> Debug (dark foreground for status)
            debug: Style::from_fg_hex(&self.palette.base04)?.dim(),
            // base05 -> Normal (default foreground)
            normal: Style::from_fg_hex(&self.palette.base05)?,
            // base06 -> Quote (light foreground)
            quote: Style::from_fg_hex(&self.palette.base06)?.italic(),
            // base07 -> Success (light background/positive)
            success: Style::from_fg_hex(&self.palette.base07)?,
            // base08 -> Error (red - variables/danger)
            error: Style::from_fg_hex(&self.palette.base08)?.bold(),
            // base09 -> Warning (orange - constants/caution)
            warning: Style::from_fg_hex(&self.palette.base09)?.bold(),
            // base0A -> Emphasis (yellow - classes/important)
            emphasis: Style::from_fg_hex(&self.palette.base0_a)?.bold(),
            // base0B -> Link (green - strings/links in some contexts)
            link: Style::from_fg_hex(&self.palette.base0_b)?.underline(),
            // base0C -> Code (cyan - support/regex)
            code: Style::from_fg_hex(&self.palette.base0_c)?.italic(),
            // base0D -> Info (blue - functions/info)
            info: Style::from_fg_hex(&self.palette.base0_d)?,
            // base0E -> Heading1 (magenta - keywords/primary)
            heading1: Style::from_fg_hex(&self.palette.base0_e)?.bold(),
            // base0F -> Heading2 (brown - deprecated/secondary)
            heading2: Style::from_fg_hex(&self.palette.base0_f)?.bold(),
            // For Base24, choose a better color for heading3 (avoid base10 which is often dark)
            heading3: Style::from_fg_hex(
                self.palette
                    .base12 // Often a bright accent color
                    .as_ref()
                    .or(self.palette.base15.as_ref()) // Fallback to another bright color
                    .or(self.palette.base16.as_ref()) // Another fallback
                    .unwrap_or(&self.palette.base0_c), // Final fallback to base16 color
            )?
            .bold(),
        })
    }

    fn create_base16_palette(&self) -> Result<Palette, Box<dyn std::error::Error>> {
        // Perfect 1:1 mapping: Base16 colors (base00-base0F) to thag roles
        Ok(Palette {
            // base00 -> Background (not a role, handled separately)
            // base01 -> Subtle (darker background/line highlighting)
            subtle: Style::from_fg_hex(&self.palette.base01)?,
            // base02 -> Commentary (selection background/comments)
            commentary: Style::from_fg_hex(&self.palette.base02)?,
            // base03 -> Hint (comments/invisibles)
            hint: Style::from_fg_hex(&self.palette.base03)?.italic(),
            // base04 -> Debug (dark foreground for status)
            debug: Style::from_fg_hex(&self.palette.base04)?.dim(),
            // base05 -> Normal (default foreground)
            normal: Style::from_fg_hex(&self.palette.base05)?,
            // base06 -> Quote (light foreground)
            quote: Style::from_fg_hex(&self.palette.base06)?.italic(),
            // base07 -> Success (light background/positive)
            success: Style::from_fg_hex(&self.palette.base07)?,
            // base08 -> Error (red - variables/danger)
            error: Style::from_fg_hex(&self.palette.base08)?.bold(),
            // base09 -> Warning (orange - constants/caution)
            warning: Style::from_fg_hex(&self.palette.base09)?.bold(),
            // base0A -> Emphasis (yellow - classes/important)
            emphasis: Style::from_fg_hex(&self.palette.base0_a)?.bold(),
            // base0B -> Link (green - strings/links in some contexts)
            link: Style::from_fg_hex(&self.palette.base0_b)?.underline(),
            // base0C -> Code (cyan - support/regex)
            code: Style::from_fg_hex(&self.palette.base0_c)?.italic(),
            // base0D -> Info (blue - functions/info)
            info: Style::from_fg_hex(&self.palette.base0_d)?,
            // base0E -> Heading1 (magenta - keywords/primary)
            heading1: Style::from_fg_hex(&self.palette.base0_e)?.bold(),
            // base0F -> Heading2 (brown - deprecated/secondary)
            heading2: Style::from_fg_hex(&self.palette.base0_f)?.bold(),
            // For Base16, we must reuse a color for heading3 since we only have 16 colors for 16 roles
            heading3: Style::from_fg_hex(&self.palette.base0_c)?.bold(), // Reuse cyan but with bold
        })
    }

    /// Enhance palette contrast for all colors with role-specific thresholds
    #[allow(clippy::too_many_lines)]
    fn enhance_palette_contrast(
        mut palette: Palette,
        background_rgb: [u8; 3],
        is_light_theme: bool,
    ) -> Palette {
        // Convert background to array for HSL conversion
        let [_bg_h, _bg_s, bg_l] = rgb_to_hsl(background_rgb);

        // Define role-specific contrast requirements
        let get_contrast_threshold = |role: &str| -> f32 {
            #[allow(clippy::match_same_arms)]
            match role {
                // Critical colors need highest contrast
                "normal" | "error" | "success" => {
                    if is_light_theme {
                        0.60
                    } else {
                        0.65
                    }
                }
                // Important colors need high contrast
                "warning" | "info" | "emphasis" | "heading1" => {
                    if is_light_theme {
                        0.55
                    } else {
                        0.60
                    }
                }
                // Secondary colors need good contrast
                "heading2" | "code" | "link" => {
                    if is_light_theme {
                        0.50
                    } else {
                        0.55
                    }
                }
                // Supporting colors - formerly problematic ones
                "subtle" | "hint" | "debug" | "commentary" => {
                    if is_light_theme {
                        0.50
                    } else {
                        0.55
                    }
                }
                // Quote can be slightly lower contrast
                "quote" => {
                    if is_light_theme {
                        0.45
                    } else {
                        0.50
                    }
                }
                // Heading3 often problematic in base24
                "heading3" => {
                    if is_light_theme {
                        0.50
                    } else {
                        0.55
                    }
                }
                _ => {
                    if is_light_theme {
                        0.45
                    } else {
                        0.50
                    }
                }
            }
        };

        // Adjust all colors for proper contrast
        palette.normal = Self::adjust_color_contrast(
            &palette.normal,
            bg_l,
            get_contrast_threshold("normal"),
            is_light_theme,
            "normal",
        );
        palette.subtle = Self::adjust_color_contrast(
            &palette.subtle,
            bg_l,
            get_contrast_threshold("subtle"),
            is_light_theme,
            "subtle",
        );
        palette.hint = Self::adjust_color_contrast(
            &palette.hint,
            bg_l,
            get_contrast_threshold("hint"),
            is_light_theme,
            "hint",
        );
        palette.debug = Self::adjust_color_contrast(
            &palette.debug,
            bg_l,
            get_contrast_threshold("debug"),
            is_light_theme,
            "debug",
        );
        palette.commentary = Self::adjust_color_contrast(
            &palette.commentary,
            bg_l,
            get_contrast_threshold("commentary"),
            is_light_theme,
            "commentary",
        );
        palette.heading1 = Self::adjust_color_contrast(
            &palette.heading1,
            bg_l,
            get_contrast_threshold("heading1"),
            is_light_theme,
            "heading1",
        );
        palette.heading2 = Self::adjust_color_contrast(
            &palette.heading2,
            bg_l,
            get_contrast_threshold("heading2"),
            is_light_theme,
            "heading2",
        );
        palette.heading3 = Self::adjust_color_contrast(
            &palette.heading3,
            bg_l,
            get_contrast_threshold("heading3"),
            is_light_theme,
            "heading3",
        );
        palette.error = Self::adjust_color_contrast(
            &palette.error,
            bg_l,
            get_contrast_threshold("error"),
            is_light_theme,
            "error",
        );
        palette.warning = Self::adjust_color_contrast(
            &palette.warning,
            bg_l,
            get_contrast_threshold("warning"),
            is_light_theme,
            "warning",
        );
        palette.success = Self::adjust_color_contrast(
            &palette.success,
            bg_l,
            get_contrast_threshold("success"),
            is_light_theme,
            "success",
        );
        palette.info = Self::adjust_color_contrast(
            &palette.info,
            bg_l,
            get_contrast_threshold("info"),
            is_light_theme,
            "info",
        );
        palette.emphasis = Self::adjust_color_contrast(
            &palette.emphasis,
            bg_l,
            get_contrast_threshold("emphasis"),
            is_light_theme,
            "emphasis",
        );
        palette.code = Self::adjust_color_contrast(
            &palette.code,
            bg_l,
            get_contrast_threshold("code"),
            is_light_theme,
            "code",
        );
        palette.link = Self::adjust_color_contrast(
            &palette.link,
            bg_l,
            get_contrast_threshold("link"),
            is_light_theme,
            "link",
        );
        palette.quote = Self::adjust_color_contrast(
            &palette.quote,
            bg_l,
            get_contrast_threshold("quote"),
            is_light_theme,
            "quote",
        );

        palette
    }

    /// Adjust a single color's contrast against the background
    fn adjust_color_contrast(
        style: &Style,
        bg_lightness: f32,
        min_lightness_diff: f32,
        is_light_theme: bool,
        role_name: &str,
    ) -> Style {
        // Extract RGB from the style
        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                ColorValue::TrueColor { rgb } => {
                    let [h, s, l] = rgb_to_hsl(*rgb);
                    let lightness_diff = (l - bg_lightness).abs();

                    // If contrast is sufficient, return as-is
                    if lightness_diff >= min_lightness_diff {
                        return style.clone();
                    }

                    // Adjust lightness for better contrast
                    let adjusted_lightness = if is_light_theme {
                        // For light themes, make text darker
                        if l > bg_lightness {
                            // Color is lighter than background, make it darker
                            (bg_lightness - min_lightness_diff).max(0.1)
                        } else {
                            // Color is darker than background, make it even darker
                            (bg_lightness - min_lightness_diff).max(0.15)
                        }
                    } else {
                        // For dark themes, make text lighter
                        if l < bg_lightness {
                            // Color is darker than background, make it lighter
                            (bg_lightness + min_lightness_diff).min(0.9)
                        } else {
                            // Color is lighter than background, make it even lighter
                            (bg_lightness + min_lightness_diff).min(0.85)
                        }
                    };

                    // Apply role-specific saturation adjustments
                    let adjusted_saturation = match role_name {
                        // Formerly problematic colors - boost significantly
                        "subtle" | "hint" | "commentary" | "debug" => (s * 1.5).min(0.8),
                        // Heading3 often needs boost in base24
                        "heading3" => (s * 1.4).min(0.8),
                        // Critical colors - moderate boost to maintain readability
                        "normal" | "error" | "success" | "warning" | "info" => (s * 1.2).min(0.9),
                        // Other colors - slight boost or maintain
                        _ => (s * 1.1).min(0.9),
                    };

                    let adjusted_rgb = hsl_to_rgb(h, adjusted_saturation, adjusted_lightness);

                    // Create new style with adjusted color, preserving other attributes
                    let mut new_style = Style::with_rgb(adjusted_rgb);
                    if style.bold {
                        new_style = new_style.bold();
                    }
                    if style.italic {
                        new_style = new_style.italic();
                    }
                    if style.dim {
                        new_style = new_style.dim();
                    }
                    if style.underline {
                        new_style = new_style.underline();
                    }

                    new_style
                }
                // For other color types, return as-is
                _ => style.clone(),
            }
        } else {
            style.clone()
        }
    }
}

#[derive(Serialize)]
struct ThemeOutput {
    name: String,
    description: String,
    term_bg_luma: String,
    min_color_support: String,
    backgrounds: Vec<String>,
    bg_rgbs: Vec<[u8; 3]>,
    base_colors: Vec<[u8; 3]>,
    palette: PaletteOutput,
}

#[derive(Serialize)]
struct PaletteOutput {
    // Headers and Structure
    heading1: StyleOutput,
    heading2: StyleOutput,
    heading3: StyleOutput,
    // Status/Alerts
    error: StyleOutput,
    warning: StyleOutput,
    success: StyleOutput,
    info: StyleOutput,
    // Emphasis levels
    emphasis: StyleOutput,
    code: StyleOutput,
    normal: StyleOutput,
    subtle: StyleOutput,
    hint: StyleOutput,
    // Development
    debug: StyleOutput,
    link: StyleOutput,
    quote: StyleOutput,
    commentary: StyleOutput,
}

#[derive(Serialize)]
struct StyleOutput {
    #[serde(flatten)]
    color: ColorOutput,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    style: Vec<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ColorOutput {
    TrueColor { rgb: [u8; 3] },
    Color256 { color256: u8 },
}

trait ToThemeOutput {
    fn to_output(&self, use_256: bool, is_base24: bool) -> ThemeOutput;
}

impl ToThemeOutput for Theme {
    fn to_output(&self, use_256: bool, is_base24: bool) -> ThemeOutput {
        let backgrounds = self.backgrounds.clone();
        let bg_rgbs = backgrounds
            .iter()
            .map(|hex| hex_to_rgb(hex).unwrap())
            .collect::<Vec<[u8; 3]>>();
        ThemeOutput {
            name: format!(
                "{}{}{}",
                self.name,
                if is_base24 { "" } else { " Base16" },
                if use_256 { " 256" } else { "" }
            ),
            description: self.description.clone(),
            term_bg_luma: self.term_bg_luma.to_string().to_lowercase(),
            min_color_support: if use_256 { "color256" } else { "true_color" }.to_string(),
            backgrounds,
            bg_rgbs,
            base_colors: self.base_colors.clone().unwrap_or_default(),
            palette: PaletteOutput {
                heading1: style_to_output(&self.palette.heading1, use_256),
                heading2: style_to_output(&self.palette.heading2, use_256),
                heading3: style_to_output(&self.palette.heading3, use_256),
                error: style_to_output(&self.palette.error, use_256),
                warning: style_to_output(&self.palette.warning, use_256),
                success: style_to_output(&self.palette.success, use_256),
                info: style_to_output(&self.palette.info, use_256),
                emphasis: style_to_output(&self.palette.emphasis, use_256),
                code: style_to_output(&self.palette.code, use_256),
                normal: style_to_output(&self.palette.normal, use_256),
                subtle: style_to_output(&self.palette.subtle, use_256),
                hint: style_to_output(&self.palette.hint, use_256),
                debug: style_to_output(&self.palette.debug, use_256),
                link: style_to_output(&self.palette.link, use_256),
                quote: style_to_output(&self.palette.quote, use_256),
                commentary: style_to_output(&self.palette.commentary, use_256),
            },
        }
    }
}

fn style_to_output(style: &Style, use_256: bool) -> StyleOutput {
    let mut style_attrs = Vec::new();
    if style.bold {
        style_attrs.push("bold".to_string());
    }
    if style.italic {
        style_attrs.push("italic".to_string());
    }
    if style.dim {
        style_attrs.push("dim".to_string());
    }
    if style.underline {
        style_attrs.push("underline".to_string());
    }

    let color = style.foreground.as_ref().map_or(
        if use_256 {
            ColorOutput::Color256 { color256: 7 }
        } else {
            ColorOutput::TrueColor {
                rgb: [192, 192, 192],
            }
        },
        |color_info| match &color_info.value {
            ColorValue::TrueColor { rgb } => {
                if use_256 {
                    ColorOutput::Color256 {
                        color256: find_closest_color(*rgb),
                    }
                } else {
                    ColorOutput::TrueColor { rgb: *rgb }
                }
            }
            ColorValue::Color256 { color256 } => ColorOutput::Color256 {
                color256: *color256,
            },
            ColorValue::Basic { .. } => {
                // Shouldn't happen for these themes, but handle gracefully
                ColorOutput::Color256 { color256: 7 } // Default to light gray
            }
        },
    );

    StyleOutput {
        color,
        style: style_attrs,
    }
}

#[derive(Parser)]
#[command(author, version, about = "Convert Base16/24 themes to thag format")]
struct Cli {
    /// Input theme file or directory
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for converted themes
    #[arg(short, long, default_value = "thag_styling/themes/built_in")]
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

fn hex_to_rgb(hex: &str) -> Result<[u8; 3], Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("Invalid hex color length".into());
    }
    Ok([
        u8::from_str_radix(&hex[0..2], 16)?,
        u8::from_str_radix(&hex[2..4], 16)?,
        u8::from_str_radix(&hex[4..6], 16)?,
    ])
}

fn detect_background_luma(hex: &str) -> Result<TermBgLuma, Box<dyn std::error::Error>> {
    let [r, g, b] = hex_to_rgb(hex)?;
    let luma =
        f32::from(b).mul_add(0.114, f32::from(r).mul_add(0.299, f32::from(g) * 0.587)) / 255.0;
    Ok(if luma > 0.5 {
        TermBgLuma::Light
    } else {
        TermBgLuma::Dark
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();

    let _ = help.check_help();

    // This will add `clap` options and exit if missing
    let cli = Cli::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&cli.output)?;

    if cli.input.is_dir() {
        eprintln!("Converting directory...");
        convert_directory(&cli)?;
    } else {
        eprintln!("Converting file...");
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
                println!("Converting {}", path.display());
            }
            convert_file(&path, cli)?;
        }
    }
    Ok(())
}

fn convert_file(input: &Path, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse YAML
    let content = fs::read_to_string(input)?;
    let base_theme: BaseTheme = serde_yaml_ok::from_str(&content)?;

    // Create output filename
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid input filename")?;

    // Convert to thag theme
    let thag_theme = base_theme.convert_to_thag()?;

    let is_base24 = base_theme.is_base24();

    // Generate TOML
    let true_color_path = cli.output.join(format!(
        "{}{}.toml",
        stem,
        if is_base24 { "" } else { "_base16" }
    ));
    if !cli.force && true_color_path.exists() {
        eprintln!("Skipping existing file: {}", true_color_path.display());
    } else {
        let theme_toml = toml::to_string_pretty(&thag_theme.to_output(false, is_base24))?; // Changed from theme to thag_theme
        fs::write(&true_color_path, theme_toml)?;
        if cli.verbose {
            println!("Created {}", true_color_path.display());
        }
    }

    // Optionally generate 256-color version
    if cli.color256 {
        let color256_path = cli.output.join(format!(
            "{}{}_256.toml",
            stem,
            if base_theme.is_base24() {
                ""
            } else {
                "_base16"
            }
        ));
        if !cli.force && color256_path.exists() {
            eprintln!("Skipping existing file: {}", color256_path.display());
        } else {
            let theme_256_toml = toml::to_string_pretty(&thag_theme.to_output(true, is_base24))?; // Changed from theme to thag_theme
            fs::write(&color256_path, theme_256_toml)?;
            if cli.verbose {
                println!("Created {}", color256_path.display());
            }
        }
    }

    Ok(())
}
