/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Converts `base16` and `base24` themes to `thag` `toml` format. Tested on `tinted-theming` crate to date.
///
/// Alternative version with headings assigned according to prominence.
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
    auto_help, find_closest_color, hsl_to_rgb, ColorSupport, ColorValue, Palette, Style,
    TermBgLuma, Theme,
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

    /// Calculate prominence score for a color based on saturation and contrast against background
    fn calculate_prominence(
        hex: &str,
        is_light_theme: bool,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        const SATURATION_WEIGHT: f32 = 0.6;
        const LIGHTNESS_WEIGHT: f32 = 0.4;

        let (r, g, b) = hex_to_rgb(hex)?;
        let (h, s, l) = Self::rgb_to_hsl([r, g, b]);
        let saturation_score = s;

        // For light themes, darker colors are more prominent
        // For dark themes, lighter colors are more prominent
        let lightness_score = if is_light_theme {
            1.0 - l // Darker = higher score
        } else {
            l // Lighter = higher score
        };

        let base_prominence =
            SATURATION_WEIGHT * saturation_score + LIGHTNESS_WEIGHT * lightness_score;

        // Apply theme-aware hue-specific adjustments based on user testing data
        let hue_multiplier = if is_light_theme {
            // Light theme adjustments: darker colors are more prominent
            match h {
                // Red/Orange: Moderate boost for darker reds
                h if h < 30.0 || h >= 330.0 => 1.08,
                // Yellow: Small reduction (bright yellows less prominent on light backgrounds)
                h if h >= 45.0 && h < 75.0 => 0.98,
                // Cyan: Small boost (dark cyans still somewhat prominent)
                h if h >= 165.0 && h < 210.0 => 1.05,
                // Blue: Small boost for darker blues
                h if h >= 210.0 && h < 270.0 => 1.05,
                // Purple: Boost for dark purples (prominent against light backgrounds)
                h if h >= 270.0 && h < 300.0 => 1.10,
                // Magenta/Pink: Moderate boost for darker magentas
                h if h >= 300.0 && h < 330.0 => 1.08,
                // Other hues: Maintain current scoring
                _ => 1.0,
            }
        } else {
            // Dark theme adjustments: brighter colors are more prominent
            match h {
                // Red/Orange: Boost significantly (user consistently ranked these higher)
                h if h < 30.0 || h >= 330.0 => 1.15,
                // Yellow/Gold: Small boost for pure yellows
                h if h >= 45.0 && h < 75.0 => 1.02,
                // Cyan/Light Blue: Boost strongly (user ranked these highest on dark themes)
                h if h >= 165.0 && h < 210.0 => 1.25,
                // Blue: Boost (user found these more prominent than algorithm calculated)
                h if h >= 210.0 && h < 270.0 => 1.10,
                // Magenta/Pink: Boost moderately for dark themes
                h if h >= 300.0 && h < 330.0 => 1.15,
                // Purple: Small reduction (user ranked lower than algorithm)
                h if h >= 270.0 && h < 300.0 => 0.95,
                // Other hues: Maintain current scoring
                _ => 1.0,
            }
        };

        Ok(base_prominence * hue_multiplier)
    }

    fn convert_to_thag(&self) -> Result<Theme, Box<dyn std::error::Error>> {
        let bg = self.palette.base00.trim_start_matches('#').to_lowercase();
        let backgrounds = vec![format!("#{bg}")];
        let bg_rgbs = vec![hex_to_rgb(&bg)?];

        let is_light_theme = matches!(
            detect_background_luma(&self.palette.base00)?,
            TermBgLuma::Light
        );
        let bg_rgb = bg_rgbs[0];

        let palette = if self.is_base24() {
            self.create_base24_palette(bg_rgb, is_light_theme)?
        } else {
            self.create_base16_palette(bg_rgb, is_light_theme)?
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
            backgrounds,
            bg_rgbs,
            is_builtin: false,
            filename: PathBuf::new(), // Will be set by caller
        })
    }

    fn create_base24_palette(
        &self,
        bg_rgb: (u8, u8, u8),
        is_light_theme: bool,
    ) -> Result<Palette, Box<dyn std::error::Error>> {
        // Create base palette without headings
        let mut palette = Palette {
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
            // Temporary placeholders for headings - will be replaced after enhancement
            heading1: Style::from_fg_hex(&self.palette.base0_e)?.bold(),
            heading2: Style::from_fg_hex(&self.palette.base0_f)?.bold(),
            heading3: Style::from_fg_hex(&self.palette.base0_c)?.bold(),
        };

        // Enhance non-heading colors for contrast
        palette = Self::enhance_palette_contrast_except_headings(palette, bg_rgb, is_light_theme);

        // Now handle heading colors: collect, enhance, then sort by prominence
        let mut heading_candidates = vec![
            (&self.palette.base0_e, "base0E"),
            (&self.palette.base0_f, "base0F"),
        ];

        // Add Base24 specific colors if available
        if let Some(ref color) = self.palette.base12 {
            heading_candidates.push((color, "base12"));
        }
        if let Some(ref color) = self.palette.base15 {
            heading_candidates.push((color, "base15"));
        }
        if let Some(ref color) = self.palette.base16 {
            heading_candidates.push((color, "base16"));
        }

        // If we still need more candidates, add base0C as fallback
        if heading_candidates.len() < 3 {
            heading_candidates.push((&self.palette.base0_c, "base0C"));
        }

        // Create and enhance heading styles
        let mut enhanced_heading_candidates: Vec<(Style, String, f32)> = heading_candidates
            .into_iter()
            .map(|(hex, name)| {
                let style = Style::from_fg_hex(hex).unwrap_or_default().bold();
                let enhanced_style = Self::enhance_single_color_contrast(
                    style,
                    bg_rgb,
                    0.60, // heading contrast threshold
                    is_light_theme,
                    "heading",
                );

                // Calculate prominence from enhanced color
                let rgb = enhanced_style
                    .foreground
                    .as_ref()
                    .and_then(|color_info| match &color_info.value {
                        thag_styling::ColorValue::TrueColor { rgb } => Some(*rgb),
                        _ => None,
                    })
                    .unwrap_or([0, 0, 0]);
                let enhanced_hex = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
                let prominence =
                    Self::calculate_prominence(&enhanced_hex, is_light_theme).unwrap_or(0.0);

                (enhanced_style, name.to_string(), prominence)
            })
            .collect();

        // Sort by prominence (most prominent first)
        enhanced_heading_candidates
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Assign headings by prominence
        palette.heading1 = enhanced_heading_candidates
            .get(0)
            .map(|x| x.0.clone())
            .unwrap_or(palette.heading1);
        palette.heading2 = enhanced_heading_candidates
            .get(1)
            .map(|x| x.0.clone())
            .unwrap_or(palette.heading2);
        palette.heading3 = enhanced_heading_candidates
            .get(2)
            .map(|x| x.0.clone())
            .unwrap_or(palette.heading3);

        Ok(palette)
    }

    fn create_base16_palette(
        &self,
        bg_rgb: (u8, u8, u8),
        is_light_theme: bool,
    ) -> Result<Palette, Box<dyn std::error::Error>> {
        // Create base palette without headings
        let mut palette = Palette {
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
            // Temporary placeholders for headings - will be replaced after enhancement
            heading1: Style::from_fg_hex(&self.palette.base0_e)?.bold(),
            heading2: Style::from_fg_hex(&self.palette.base0_f)?.bold(),
            heading3: Style::from_fg_hex(&self.palette.base0_c)?.bold(),
        };

        // Enhance non-heading colors for contrast
        palette = Self::enhance_palette_contrast_except_headings(palette, bg_rgb, is_light_theme);

        // Now handle heading colors: collect, enhance, then sort by prominence
        let heading_candidates = vec![
            (&self.palette.base0_e, "base0E"),
            (&self.palette.base0_f, "base0F"),
            (&self.palette.base0_c, "base0C"), // Must reuse for Base16
        ];

        // Create and enhance heading styles
        let mut enhanced_heading_candidates: Vec<(Style, String, f32)> = heading_candidates
            .into_iter()
            .map(|(hex, name)| {
                let style = Style::from_fg_hex(hex).unwrap_or_default().bold();
                let enhanced_style = Self::enhance_single_color_contrast(
                    style,
                    bg_rgb,
                    0.60, // heading contrast threshold
                    is_light_theme,
                    "heading",
                );

                // Calculate prominence from enhanced color
                let rgb = enhanced_style
                    .foreground
                    .as_ref()
                    .and_then(|color_info| match &color_info.value {
                        thag_styling::ColorValue::TrueColor { rgb } => Some(*rgb),
                        _ => None,
                    })
                    .unwrap_or([0, 0, 0]);
                let enhanced_hex = format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]);
                let prominence =
                    Self::calculate_prominence(&enhanced_hex, is_light_theme).unwrap_or(0.0);

                (enhanced_style, name.to_string(), prominence)
            })
            .collect();

        // Sort by prominence (most prominent first)
        enhanced_heading_candidates
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Assign headings by prominence
        palette.heading1 = enhanced_heading_candidates[0].0.clone();
        palette.heading2 = enhanced_heading_candidates[1].0.clone();
        palette.heading3 = enhanced_heading_candidates[2].0.clone();

        Ok(palette)
    }

    /// Convert RGB to HSL color space
    #[allow(clippy::many_single_char_names)]
    fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
        let r = f32::from(rgb[0]) / 255.0;
        let g = f32::from(rgb[1]) / 255.0;
        let b = f32::from(rgb[2]) / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;
        let (s, mut h) = if delta == 0.0 {
            (0.0, 0.0)
        } else {
            let s = if l > 0.5 {
                delta / (2.0 - max - min)
            } else {
                delta / (max + min)
            };

            let h = if (max - r).abs() < f32::EPSILON {
                ((g - b) / delta) % 6.0
            } else if (max - g).abs() < f32::EPSILON {
                ((b - r) / delta) + 2.0
            } else {
                ((r - g) / delta) + 4.0
            } * 60.0;

            (s, h)
        };

        // Ensure hue is positive
        if h < 0.0 {
            h += 360.0;
        }

        (h, s, l)
    }

    /// Enhance palette contrast for all colors except headings (handled separately)
    #[allow(clippy::too_many_lines)]
    fn enhance_palette_contrast_except_headings(
        mut palette: Palette,
        background_rgb: (u8, u8, u8),
        is_light_theme: bool,
    ) -> Palette {
        // Convert background to array for HSL conversion
        let bg_array = [background_rgb.0, background_rgb.1, background_rgb.2];
        let (_bg_h, _bg_s, bg_l) = Self::rgb_to_hsl(bg_array);

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
                // Supporting colors - formerly problematic ones (increased contrast for selection visibility)
                "subtle" | "commentary" => {
                    if is_light_theme {
                        0.60
                    } else {
                        0.70
                    }
                }
                // Other supporting colors
                "hint" | "debug" => {
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
        // Headings are handled separately in the create_*_palette functions
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

    /// Enhance a single color for contrast with role-specific handling
    fn enhance_single_color_contrast(
        style: Style,
        background_rgb: (u8, u8, u8),
        contrast_threshold: f32,
        is_light_theme: bool,
        role_name: &str,
    ) -> Style {
        let bg_array = [background_rgb.0, background_rgb.1, background_rgb.2];
        let (_bg_h, _bg_s, bg_l) = Self::rgb_to_hsl(bg_array);

        Self::adjust_color_contrast(&style, bg_l, contrast_threshold, is_light_theme, role_name)
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
                    let (h, s, l) = Self::rgb_to_hsl(*rgb);
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
    bg_rgbs: Vec<(u8, u8, u8)>, // Official one first
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
            .collect::<Vec<(u8, u8, u8)>>();
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
                        color256: find_closest_color((rgb[0], rgb[1], rgb[2])),
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
    let theme = base_theme.convert_to_thag()?;

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
        let theme_toml = toml::to_string_pretty(&theme.to_output(false, is_base24))?;
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
            let theme_256_toml = toml::to_string_pretty(&theme.to_output(true, is_base24))?;
            fs::write(&color256_path, theme_256_toml)?;
            if cli.verbose {
                println!("Created {}", color256_path.display());
            }
        }
    }

    Ok(())
}
