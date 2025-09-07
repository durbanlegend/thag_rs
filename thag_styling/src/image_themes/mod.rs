//! Image-based theme generation for extracting color palettes from images
//!
//! This module provides functionality to analyze images and generate terminal color themes
//! based on the dominant colors found in the image. It uses color clustering to identify
//! the most prominent colors and intelligently maps them to semantic roles.

// #![cfg(feature = "image_themes")]

use crate::{
    cvprtln,
    styling::{self, rgb_to_hex},
    vprtln, ColorSupport, Palette, Role, Style, StylingError, StylingResult, TermBgLuma, Theme, V,
};
use image::{DynamicImage, ImageReader};
use palette::{FromColor, Hsl, IntoColor, Lab, Srgb};
use std::fmt::Write as _; // import without risk of name clashing
use std::path::{Path, PathBuf};

/// Struct to hold semantic color assignments
struct SemanticColors<'a> {
    error: &'a ColorAnalysis,
    warning: &'a ColorAnalysis,
    success: &'a ColorAnalysis,
    info: &'a ColorAnalysis,
    code: &'a ColorAnalysis,
    emphasis: &'a ColorAnalysis,
}

/// Configuration for image theme generation
#[derive(Debug, Clone)]
pub struct ImageThemeConfig {
    /// Number of dominant colors to extract (default: 16)
    pub color_count: usize,
    /// Minimum brightness threshold for light colors (0.0-1.0, default: 0.7)
    pub light_threshold: f32,
    /// Minimum saturation threshold for accent colors (0.0-1.0, default: 0.3)
    pub saturation_threshold: f32,
    /// Whether to automatically determine if theme should be light or dark
    pub auto_detect_theme_type: bool,
    /// Force theme to be light or dark (overrides auto-detection)
    pub force_theme_type: Option<TermBgLuma>,
    /// Name prefix for the generated theme
    pub theme_name_prefix: Option<String>,
}

impl Default for ImageThemeConfig {
    fn default() -> Self {
        Self {
            color_count: 16,
            light_threshold: 0.7,
            saturation_threshold: 0.3,
            auto_detect_theme_type: true,
            force_theme_type: None,
            theme_name_prefix: None,
        }
    }
}

/// Represents a color with its HSL properties for easier analysis
#[derive(Debug, Clone, PartialEq)]
struct ColorAnalysis {
    rgb: [u8; 3],
    lab: Lab,
    hue: f32,
    saturation: f32,
    lightness: f32,
    frequency: f32,
}

impl ColorAnalysis {
    fn new(rgb: [u8; 3], frequency: f32) -> Self {
        let (lab, hsl) = to_lab_hsl(rgb);

        Self {
            rgb,
            lab,
            hue: hsl.hue.into_positive_degrees(),
            saturation: hsl.saturation,
            lightness: hsl.lightness,
            frequency,
        }
    }

    // /// Check if this color is suitable as a background color
    // fn is_background_suitable(&self) -> bool {
    //     // dbg!(self.saturation < 0.2 && (self.lightness < 0.15 || self.lightness > 0.85));
    //     // Background colors should be neutral (low saturation) and either very light or very dark
    //     let is_background_suitable =
    //         self.saturation < 0.2 && (self.lightness < 0.15 || self.lightness > 0.85);
    //     // dbg!(is_background_suitable);
    //     vprtln!(V::V,
    //         "self.rgb={},{},{}, is_background_suitable={is_background_suitable}",
    //         self.rgb[0], self.rgb[1], self.rgb[2]
    //     );
    //     is_background_suitable
    // }

    /// Check if this color is suitable for text (good contrast potential)
    fn is_text_suitable(&self, is_light_theme: bool) -> bool {
        let is_text_suitable = if is_light_theme {
            // For light themes, text should be dark but not too dark (avoid pure black)
            self.lightness > 0.25 && self.lightness < 0.6
        } else {
            // For dark themes, text should be light but not too light (avoid pure white)
            self.lightness > 0.6 && self.lightness < 0.75
        };
        vprtln!(V::V,
            "is_light_theme={is_light_theme}, self.rgb={}, self.lightness={}, is_text_suitable={is_text_suitable}",
            Style::new().with_rgb(self.rgb).paint(format!("{:?}", self.rgb)), self.lightness
        );
        // dbg!(is_text_suitable);
        is_text_suitable
    }

    /// Check contrast against background
    fn has_good_contrast_against(&self, background: &Self) -> bool {
        let lightness_diff = (self.lightness - background.lightness).abs();
        // dbg!(lightness_diff);
        let has_good_contrast_against = lightness_diff > 0.4; // Minimum contrast requirement only, no upper limit
                                                              // dbg!(has_good_contrast_against);
        vprtln!(V::V, "self.rgb={}, background.lightness={}, lightness_diff={lightness_diff}, has_good_contrast_against={has_good_contrast_against}", Style::new().with_rgb(self.rgb).paint(format!("{:?}", self.rgb)), background.lightness);
        has_good_contrast_against
    }

    /// Calculate perceptual distance to another color using Delta E
    fn distance_to(&self, other: &Self) -> f32 {
        let delta_l = self.lab.l - other.lab.l;
        let delta_a = self.lab.a - other.lab.a;
        let delta_b = self.lab.b - other.lab.b;

        // Simplified Delta E calculation
        delta_b
            .mul_add(delta_b, delta_l.mul_add(delta_l, delta_a * delta_a))
            .sqrt()
    }
}

fn to_lab_hsl(rgb: [u8; 3]) -> (Lab, Hsl) {
    let srgb = Srgb::new(
        f32::from(rgb[0]) / 255.0,
        f32::from(rgb[1]) / 255.0,
        f32::from(rgb[2]) / 255.0,
    );

    let lab: Lab = srgb.into_color();
    let hsl: Hsl = Hsl::from_color(srgb);
    (lab, hsl)
}

/// Theme generator from images
pub struct ImageThemeGenerator {
    config: ImageThemeConfig,
}

impl ImageThemeGenerator {
    /// Create a new image theme generator with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ImageThemeConfig::default(),
        }
    }

    /// Create a new image theme generator with custom configuration
    #[must_use]
    pub const fn with_config(config: ImageThemeConfig) -> Self {
        Self { config }
    }

    /// Generate a theme from an image file
    ///
    /// # Errors
    ///
    /// This function will bubble up any errors encountered.
    pub fn generate_from_file<P: AsRef<Path>>(&self, image_path: P) -> StylingResult<Theme> {
        let image = ImageReader::open(&image_path)
            .map_err(|e| StylingError::Generic(format!("Failed to open image: {e}")))?
            .decode()
            .map_err(|e| StylingError::Generic(format!("Failed to decode image: {e}")))?;

        let theme_name = self.generate_theme_name(&image_path);
        self.generate_from_image(&image, theme_name)
    }

    /// Generate a theme from a loaded image
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o errors encountered.
    pub fn generate_from_image(
        &self,
        image: &DynamicImage,
        theme_name: String,
    ) -> StylingResult<Theme> {
        let dominant_colors = self.extract_dominant_colors(image)?;
        cvprtln!(Role::HD1, V::V, "Dominant colors:");
        for (color, freq) in &dominant_colors {
            let (__lab, hsl) = to_lab_hsl(*color);
            vprtln!(
                V::V,
                "{} with frequency {freq:.3}",
                Style::new().with_rgb(*color).paint(format!(
                    "{color:?} = hue: {:.0}",
                    hsl.hue.into_positive_degrees()
                ))
            );
        }

        let color_analysis = Self::analyze_colors(dominant_colors);

        let is_light_theme = self.determine_theme_type(&color_analysis);
        let background_color = Self::select_background_color(&color_analysis, is_light_theme);
        vprtln!(
            V::V,
            "Selected background color={} ({:?})",
            Style::new()
                .with_rgb(background_color.rgb)
                .paint(format!("{:?}", background_color.rgb)),
            background_color.rgb
        );

        let palette = Self::map_colors_to_roles(&background_color, &color_analysis, is_light_theme);

        Ok(Theme {
            name: theme_name,
            filename: PathBuf::from("generated.toml"),
            is_builtin: false,
            description: "Generated from image analysis".to_string(),
            term_bg_luma: if is_light_theme {
                TermBgLuma::Light
            } else {
                TermBgLuma::Dark
            },
            min_color_support: ColorSupport::TrueColor,
            backgrounds: vec![format!(
                "#{:02x}{:02x}{:02x}",
                background_color.rgb[0], background_color.rgb[1], background_color.rgb[2]
            )],
            bg_rgbs: vec![(
                background_color.rgb[0],
                background_color.rgb[1],
                background_color.rgb[2],
            )],
            palette,
        })
    }

    /// Extract dominant colors from an image ensuring diversity and contrast
    #[allow(clippy::cast_precision_loss)]
    fn extract_dominant_colors(&self, image: &DynamicImage) -> StylingResult<Vec<([u8; 3], f32)>> {
        let rgb_image = image.to_rgb8();
        let pixels: Vec<[u8; 3]> = rgb_image.pixels().map(|p| [p[0], p[1], p[2]]).collect();

        if pixels.is_empty() {
            return Err(StylingError::Generic(
                "Image contains no pixels".to_string(),
            ));
        }

        // More aggressive quantization to reduce similar colors
        let mut color_counts: std::collections::HashMap<[u8; 3], usize> =
            std::collections::HashMap::new();

        // Quantize colors with more aggressive reduction
        for pixel in &pixels {
            let quantized = [
                (pixel[0] / 24) * 24, // Reduce to ~10 levels per channel
                (pixel[1] / 24) * 24,
                (pixel[2] / 24) * 24,
            ];
            *color_counts.entry(quantized).or_insert(0) += 1;
        }

        // Sort colors by frequency
        let mut colors_by_frequency: Vec<_> = color_counts.into_iter().collect();
        colors_by_frequency.sort_by(|a, b| b.1.cmp(&a.1));

        let total_pixels = pixels.len() as f32;
        let mut result = Vec::new();

        // Select diverse colors with minimum distance requirement
        for (color, count) in colors_by_frequency {
            let frequency = count as f32 / total_pixels;

            // Check if this color is sufficiently different from already selected colors
            let is_diverse = result.is_empty()
                || result.iter().all(|(existing_color, _)| {
                    Self::color_distance_euclidean(*existing_color, color) > 60.0
                    // Minimum distance threshold
                });

            if is_diverse {
                result.push((color, frequency));
                if result.len() >= self.config.color_count {
                    break;
                }
            }
        }

        // Ensure we have good color diversity - add contrasting colors if needed
        Self::ensure_color_diversity(&mut result, total_pixels);

        Ok(result)
    }

    /// Calculate Euclidean distance between two RGB colors
    fn color_distance_euclidean(color1: [u8; 3], color2: [u8; 3]) -> f32 {
        let dr = f32::from(color1[0]) - f32::from(color2[0]);
        let dg = f32::from(color1[1]) - f32::from(color2[1]);
        let db = f32::from(color1[2]) - f32::from(color2[2]);
        db.mul_add(db, dr.mul_add(dr, dg * dg)).sqrt()
    }

    /// Ensure extracted colors have good diversity by adding contrasting colors if needed
    fn ensure_color_diversity(colors: &mut Vec<([u8; 3], f32)>, _total_pixels: f32) {
        let min_colors = 8;

        if colors.len() >= min_colors {
            return;
        }

        // Find the lowest existing frequency among dominants
        let min_freq = colors
            .iter()
            .map(|(_, freq)| *freq)
            .fold(f32::INFINITY, f32::min);
        let artificial_freq = if min_freq.is_finite() {
            (min_freq * 0.9).max(0.0001) // just under the smallest real freq
        } else {
            0.0001
        };

        Self::generate_more_colors(colors, min_colors, artificial_freq);
    }

    fn generate_more_colors(
        colors: &mut Vec<([u8; 3], f32)>,
        min_colors: usize,
        artificial_freq: f32,
    ) {
        // Go through existing colours in order
        let mut idx = 0;
        while colors.len() < min_colors && idx < colors.len() {
            let base_rgb = colors[idx].0;
            let (h, s, l) = rgb_to_hsl(base_rgb);

            // lighter
            let lighter_l = (l + 0.15).min(1.0);
            let lighter = hsl_to_rgb(h, s, lighter_l);
            if colors
                .iter()
                .all(|(c, _)| Self::color_distance_euclidean(*c, lighter) > 20.0)
            {
                vprtln!(
                    V::V,
                    "New color {}",
                    Style::new().with_rgb(lighter).paint(format!("{lighter:?}"))
                );
                colors.push((lighter, artificial_freq));
                if colors.len() >= min_colors {
                    break;
                }
            }

            // darker
            let darker_l = (l - 0.15).max(0.0);
            let darker = hsl_to_rgb(h, s, darker_l);
            if colors
                .iter()
                .all(|(c, _)| Self::color_distance_euclidean(*c, darker) > 20.0)
            {
                vprtln!(
                    V::V,
                    "New color {}",
                    Style::new().with_rgb(darker).paint(format!("{darker:?}"))
                );
                colors.push((darker, artificial_freq));
            }

            idx += 1;
        }
    }

    /// Analyze colors and create `ColorAnalysis` structures
    fn analyze_colors(colors: Vec<([u8; 3], f32)>) -> Vec<ColorAnalysis> {
        colors
            .into_iter()
            .map(|(rgb, freq)| ColorAnalysis::new(rgb, freq))
            .collect()
    }

    /// Determine if the theme should be light or dark
    fn determine_theme_type(&self, colors: &[ColorAnalysis]) -> bool {
        // dbg!(self.config.force_theme_type);
        if let Some(forced_type) = &self.config.force_theme_type {
            return *forced_type == TermBgLuma::Light;
        }

        // dbg!(self.config.auto_detect_theme_type);

        if !self.config.auto_detect_theme_type {
            // Default to dark theme if not auto-detecting
            return false;
        }

        // Calculate weighted average lightness of all colors
        let total_weight: f32 = colors.iter().map(|c| c.frequency).sum();
        // dbg!(total_weight);
        if total_weight == 0.0 {
            return false;
        }

        let weighted_lightness: f32 = colors
            .iter()
            .map(|c| c.lightness * c.frequency)
            .sum::<f32>()
            / total_weight;

        // dbg!(weighted_lightness);
        // dbg!(self.config.light_threshold);

        // Theme is light if average lightness is above threshold
        weighted_lightness > self.config.light_threshold
    }

    /// Adjust color contrast against background, with saturation adjustments optimized for theme type
    fn adjust_color_contrast(
        color: &ColorAnalysis,
        background: &ColorAnalysis,
        min_lightness_diff: f32,
        is_light_theme: bool,
        adjust_saturation: bool,
        color_name: &str,
    ) -> ColorAnalysis {
        let mut adjusted_lightness = color.lightness;
        let mut lightness_diff = (adjusted_lightness - background.lightness).abs();

        // Adjust lightness to meet minimum contrast requirement
        while lightness_diff < min_lightness_diff {
            if is_light_theme {
                if adjusted_lightness > background.lightness {
                    // Color is lighter than background, make it lighter
                    adjusted_lightness = (adjusted_lightness * 1.05).min(0.95);
                } else {
                    // Color is darker than background, make it darker
                    adjusted_lightness = (adjusted_lightness / 1.05).max(0.05);
                }
            } else {
                if adjusted_lightness > background.lightness {
                    // Color is lighter than background, make it lighter
                    adjusted_lightness = (adjusted_lightness * 1.05).min(0.95);
                } else {
                    // Color is darker than background, make it darker
                    adjusted_lightness = (adjusted_lightness / 1.05).max(0.05);
                }
            }
            lightness_diff = (adjusted_lightness - background.lightness).abs();
        }

        // Adjust saturation based on theme type
        let adjusted_saturation = if adjust_saturation {
            if is_light_theme {
                // Boost saturation for light themes to preserve vibrancy when darkening colors
                (color.saturation * 1.3).min(0.95)
            } else {
                // Reduce saturation for dark themes for better readability
                (color.saturation * 0.7).max(0.05)
            }
        } else {
            // For semantic/core colors, apply light theme boost to maintain vibrancy
            if is_light_theme {
                (color.saturation * 1.1).min(0.95)
            } else {
                color.saturation
            }
        };

        let rgb = hsl_to_rgb(color.hue, adjusted_saturation, adjusted_lightness);

        // Debug output
        println!(
            "{}: {}",
            color_name,
            Style::new().with_rgb(rgb).paint(format!(
                "lightness_diff={:.3}, rgb={}",
                lightness_diff,
                rgb_to_hex(&rgb.into())
            ))
        );

        ColorAnalysis::new(rgb, 0.0)
    }

    /// Map extracted colors to semantic roles with improved contrast and diversity
    #[allow(clippy::too_many_lines)]
    fn map_colors_to_roles(
        background_color: &ColorAnalysis,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> styling::Palette {
        // Find suitable colors for different categories with better filtering
        let text_colors: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_text_suitable(is_light_theme))
            .collect();

        // let _accent_colors: Vec<&ColorAnalysis> = colors
        //     .iter()
        //     .filter(|c| c.is_accent_suitable(self.config.saturation_threshold))
        //     .collect();

        // If we don't have enough diverse colors, create synthetic ones
        let enhanced_colors = Self::enhance_color_palette(colors);

        let enhanced_colors: Vec<ColorAnalysis> = enhanced_colors
            .iter()
            .filter(|color| color.has_good_contrast_against(background_color))
            .cloned()
            .collect();

        cvprtln!(Role::HD1, V::V, "Selected colors:");
        for color in &enhanced_colors {
            vprtln!(
                V::V,
                "{}",
                Style::new().with_rgb(color.rgb).paint(format!(
                    "{} {:?} = hue: {:.0}",
                    rgb_to_hex(&color.rgb.into()),
                    color.rgb,
                    color.hue
                ))
            );
        }

        let normal_color = Self::select_best_text_color(
            &text_colors,
            &enhanced_colors,
            is_light_theme,
            Some(background_color),
        )
        .map_or_else(
            || {
                // Ensure we have a proper text color with good contrast
                vprtln!(V::V, "Calling ensure_text_contrast");
                Self::ensure_text_contrast(&enhanced_colors, background_color, is_light_theme)
            },
            |best_text| {
                if best_text.distance_to(background_color) < 50.0 {
                    vprtln!(V::V, "Distance < 50, calling ensure_text_contrast");
                    Self::ensure_text_contrast(&enhanced_colors, background_color, is_light_theme)
                } else {
                    vprtln!(V::V, "Going with best_text");
                    best_text
                }
            },
        );
        // dbg!(normal_color);

        // Create a comprehensive unique color assignment
        let mut used_colors = vec![normal_color];

        // let subtle_color = self.find_most_different_color(&enhanced_colors, &used_colors);
        // used_colors.push(subtle_color);

        // let hint_color = self.find_most_different_color(&enhanced_colors, &used_colors);
        // used_colors.push(hint_color);

        // Select heading colors with good contrast and uniqueness
        let heading_colors = Self::select_unique_heading_colors(&enhanced_colors, &used_colors);
        let (hd1, hd2, hd3) = heading_colors;
        vprtln!(
            V::V,
            "Heading1={}",
            Style::new().with_rgb(hd1.rgb).bold().paint(format!(
                "{}, hue: {}, saturation: {}, lightness: {}",
                rgb_to_hex(&hd1.rgb.into()),
                hd1.hue,
                hd1.saturation,
                hd1.lightness
            ))
        );
        vprtln!(
            V::V,
            "Heading2={}",
            Style::new().with_rgb(hd2.rgb).bold().paint(format!(
                "{}, hue: {}, saturation: {}, lightness: {}",
                rgb_to_hex(&hd2.rgb.into()),
                hd2.hue,
                hd2.saturation,
                hd2.lightness
            ))
        );
        vprtln!(
            V::V,
            "Heading3={}",
            Style::new().with_rgb(hd3.rgb).bold().paint(format!(
                "{}, hue: {}, saturation: {}, lightness: {}",
                rgb_to_hex(&hd3.rgb.into()),
                hd3.hue,
                hd3.saturation,
                hd3.lightness
            ))
        );
        used_colors.push(hd1);
        used_colors.push(hd2);
        used_colors.push(hd3);

        let mut used_colors_clone = used_colors.clone();

        // Map colors to semantic roles with better contrast and diversity
        let semantic_colors = Self::assign_semantic_colors(
            &enhanced_colors,
            &mut used_colors_clone,
            normal_color,
            is_light_theme,
        );

        // Ensure semantic colors are also unique from normal/subtle
        used_colors.extend(&[
            semantic_colors.error,
            semantic_colors.warning,
            semantic_colors.success,
            semantic_colors.info,
            semantic_colors.code,
            semantic_colors.emphasis,
        ]);

        vprtln!(V::V, "");
        assert!(used_colors.contains(&semantic_colors.error));
        let subtle_color =
            Self::find_most_different_color(&enhanced_colors, &used_colors[..], background_color);
        vprtln!(
            V::V,
            "subtle_color={}",
            Style::new().with_rgb(subtle_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&subtle_color.rgb.into()),
                subtle_color.hue
            ))
        );

        // Apply contrast adjustment to subtle color
        let adjusted_subtle_color = Self::adjust_color_contrast(
            subtle_color,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Subtle",
        );

        used_colors.push(subtle_color);

        let hint_color =
            Self::find_most_different_color(&enhanced_colors, &used_colors[..], background_color);
        vprtln!(
            V::V,
            "hint_color={}",
            Style::new().with_rgb(hint_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&hint_color.rgb.into()),
                hint_color.hue
            ))
        );

        // Apply contrast adjustment to hint color
        let adjusted_hint_color = Self::adjust_color_contrast(
            hint_color,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Hint",
        );

        used_colors.push(hint_color);

        // Debug and trace should be different from subtle and hint
        let debug_color =
            Self::find_most_different_color(&enhanced_colors, &used_colors[..], background_color);
        vprtln!(
            V::V,
            "debug_color={}",
            Style::new().with_rgb(debug_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&debug_color.rgb.into()),
                debug_color.hue
            ))
        );

        // Apply contrast adjustment to debug color
        let adjusted_debug_color = Self::adjust_color_contrast(
            debug_color,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Debug",
        );

        // Derive three distinct colors for the new roles from existing palette colors
        used_colors.push(debug_color);

        // Apply contrast adjustment to semantic colors (0.7 minimum for better readability)
        let adjusted_error = Self::adjust_color_contrast(
            semantic_colors.error,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Error",
        );
        let adjusted_warning = Self::adjust_color_contrast(
            semantic_colors.warning,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Warning",
        );
        let adjusted_success = Self::adjust_color_contrast(
            semantic_colors.success,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Success",
        );
        let adjusted_info = Self::adjust_color_contrast(
            semantic_colors.info,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Info",
        );
        let adjusted_code = Self::adjust_color_contrast(
            semantic_colors.code,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Code",
        );
        let adjusted_emphasis = Self::adjust_color_contrast(
            semantic_colors.emphasis,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Emphasis",
        );

        // Link color: derive from error color (typically red/bright for visibility)
        let link_color = Self::adjust_color_contrast(
            &adjusted_error,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Link",
        );

        // Quote color: derive from subtle color with reduced saturation for muted appearance
        let quote_color = Self::adjust_color_contrast(
            &adjusted_subtle_color,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Quote",
        );

        // Commentary color: derive from normal color
        let commentary_color = Self::adjust_color_contrast(
            &normal_color,
            background_color,
            0.6,
            is_light_theme,
            true,
            "Commentary",
        );

        vprtln!(
            V::V,
            "link_color={}",
            Style::new().with_rgb(link_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&link_color.rgb.into()),
                link_color.hue
            ))
        );
        vprtln!(
            V::V,
            "quote_color={}",
            Style::new().with_rgb(quote_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&quote_color.rgb.into()),
                quote_color.hue
            ))
        );
        vprtln!(
            V::V,
            "commentary_color={}",
            Style::new().with_rgb(commentary_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&commentary_color.rgb.into()),
                commentary_color.hue
            ))
        );

        // Apply contrast adjustment to heading colors
        let adjusted_heading_colors = (
            Self::adjust_color_contrast(
                &heading_colors.0,
                background_color,
                0.7,
                is_light_theme,
                false,
                "Heading1",
            ),
            Self::adjust_color_contrast(
                &heading_colors.1,
                background_color,
                0.7,
                is_light_theme,
                false,
                "Heading2",
            ),
            Self::adjust_color_contrast(
                &heading_colors.2,
                background_color,
                0.7,
                is_light_theme,
                false,
                "Heading3",
            ),
        );

        // Apply contrast adjustment to normal color
        let adjusted_normal_color = Self::adjust_color_contrast(
            &normal_color,
            background_color,
            0.7,
            is_light_theme,
            false,
            "Normal",
        );

        Palette {
            normal: Style::new().with_rgb(adjusted_normal_color.rgb),
            subtle: Style::new().with_rgb(adjusted_subtle_color.rgb),
            hint: Style::new().with_rgb(adjusted_hint_color.rgb).italic(),
            heading1: Style::new().with_rgb(adjusted_heading_colors.0.rgb).bold(),
            heading2: Style::new().with_rgb(adjusted_heading_colors.1.rgb).bold(),
            heading3: Style::new().with_rgb(adjusted_heading_colors.2.rgb).bold(),
            error: Style::new().with_rgb(adjusted_error.rgb),
            warning: Style::new().with_rgb(adjusted_warning.rgb),
            success: Style::new().with_rgb(adjusted_success.rgb),
            info: Style::new().with_rgb(adjusted_info.rgb),
            code: Style::new().with_rgb(adjusted_code.rgb),
            emphasis: Style::new().with_rgb(adjusted_emphasis.rgb),
            debug: Style::new().with_rgb(adjusted_debug_color.rgb).italic(),
            link: Style::new().with_rgb(link_color.rgb).underline(),
            quote: Style::new().with_rgb(quote_color.rgb).italic(),
            commentary: Style::new().with_rgb(commentary_color.rgb).italic(),
        }
    }

    /// Enhance color palette with derived colors if diversity is lacking
    fn enhance_color_palette(colors: &[ColorAnalysis]) -> Vec<ColorAnalysis> {
        let mut enhanced = colors.to_vec();

        Self::generate_derived_colors(&mut enhanced);

        enhanced
    }

    /// Generate derived colors to improve palette diversity
    fn generate_derived_colors(colors: &mut Vec<ColorAnalysis>) {
        // Go through existing colours in order
        let mut idx = 0;
        let min_colors = colors.len() * 2;
        let mut new_count = 0;
        let adjust_perc = 0.10;

        while new_count < min_colors && idx < colors.len() {
            vprtln!(
                V::V,
                "Existing color {}",
                Style::new()
                    .with_rgb(colors[idx].rgb)
                    .paint(format!("{:?}", colors[idx].rgb))
            );

            let l = colors[idx].lightness;

            // lighter
            let lighter_l = (l + adjust_perc).min(1.0);
            let lighter = hsl_to_rgb(colors[idx].hue, colors[idx].saturation, lighter_l);
            if colors
                .iter()
                .map(|color| color.rgb)
                .all(|rgb| Self::color_distance_euclidean(rgb, lighter) > 20.0)
            {
                vprtln!(
                    V::V,
                    "New color {}",
                    Style::new().with_rgb(lighter).paint(format!("{lighter:?}"))
                );
                colors.push(ColorAnalysis::new(lighter, 0.0));
                new_count += 1;
                if new_count >= min_colors {
                    break;
                }
            }

            // darker
            let darker_l = (l - adjust_perc).max(0.0);
            let darker = hsl_to_rgb(colors[idx].hue, colors[idx].saturation, darker_l);
            if colors
                .iter()
                .map(|color| color.rgb)
                .all(|rgb| Self::color_distance_euclidean(rgb, darker) > 20.0)
            {
                vprtln!(
                    V::V,
                    "New color {}",
                    Style::new().with_rgb(darker).paint(format!("{darker:?}"))
                );
                colors.push(ColorAnalysis::new(darker, 0.0));
                new_count += 1;
                if new_count >= min_colors {
                    break;
                }
            }

            idx += 1;
        }
    }

    /// Select the best text color with contrast consideration
    fn select_best_text_color<'a>(
        text_colors: &[&'a ColorAnalysis],
        all_colors: &'a [ColorAnalysis],
        is_light_theme: bool,
        background: Option<&ColorAnalysis>,
    ) -> Option<&'a ColorAnalysis> {
        // First try to find text colors with good background contrast
        if let Some(bg) = background {
            if let Some(best) = text_colors
                .iter()
                .filter(|c| c.has_good_contrast_against(bg))
                .max_by(|a, b| {
                    a.frequency
                        .partial_cmp(&b.frequency)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                vprtln!(
                    V::V,
                    "Found best text color {}",
                    Style::new()
                        .with_rgb(best.rgb)
                        .paint(format!("{:?}", best.rgb))
                );
                return Some(*best);
            }
        }

        // Fall back to any suitable text color
        let best_text = text_colors
            .iter()
            .max_by(|a, b| {
                a.frequency
                    .partial_cmp(&b.frequency)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map_or_else(
                || {
                    // Final fallback: find any color with good contrast
                    vprtln!(V::V, "Falling back to any color with good contrast");
                    all_colors
                        .iter()
                        .find(|c| c.is_text_suitable(is_light_theme))
                },
                |best| {
                    vprtln!(
                        V::V,
                        "Falling back to text color {}",
                        Style::new()
                            .with_rgb(best.rgb)
                            .paint(format!("{:?}", best.rgb))
                    );
                    Some(*best)
                },
            );
        vprtln!(
            V::V,
            "Selected best_text {}",
            Style::new()
                .with_rgb(best_text.unwrap().rgb)
                .paint(format!("{:?}", best_text.unwrap().rgb))
        );
        best_text
    }

    /// Ensure we have a text color with proper contrast
    fn ensure_text_contrast<'a>(
        colors: &'a [ColorAnalysis],
        background: &ColorAnalysis,
        is_light_theme: bool,
    ) -> &'a ColorAnalysis {
        // First, find colors with good contrast that are also different enough
        if let Some(good_contrast) = colors
            .iter()
            .filter(|c| c.has_good_contrast_against(background) && c.distance_to(background) > 50.0)
            .max_by(|a, b| {
                let contrast_a = (a.lightness - background.lightness).abs();
                let contrast_b = (b.lightness - background.lightness).abs();
                contrast_a
                    .partial_cmp(&contrast_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            vprtln!(V::V, "Found good contrast: {:?}", good_contrast.rgb);
            return good_contrast;
        }

        vprtln!(V::V, "Falling back to synthetic text color");

        // If no good contrast found, create synthetic text color
        // For light theme background, we need a darker text
        // For dark theme background, we need a lighter text
        if is_light_theme {
            // Find the darkest available color that's not too close to background
            colors
                .iter()
                .filter(|c| c.lightness < 0.5 && c.distance_to(background) > 30.0)
                .min_by(|a, b| {
                    a.lightness
                        .partial_cmp(&b.lightness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or_else(|| {
                    // Ultimate fallback: find any dark-ish color
                    colors
                        .iter()
                        .min_by(|a, b| {
                            a.lightness
                                .partial_cmp(&b.lightness)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap_or(&colors[0])
                })
        } else {
            // Find the lightest available color that's not too close to background
            colors
                .iter()
                .filter(|c| c.lightness > 0.5 && c.distance_to(background) > 30.0)
                .max_by(|a, b| {
                    a.lightness
                        .partial_cmp(&b.lightness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or_else(|| {
                    // Ultimate fallback: find any light-ish color
                    colors
                        .iter()
                        .max_by(|a, b| {
                            a.lightness
                                .partial_cmp(&b.lightness)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap_or(&colors[0])
                })
        }
    }

    // /// Select subtle color with proper contrast relationship
    // fn select_subtle_color<'a>(
    //     &self,
    //     colors: &'a [ColorAnalysis],
    //     normal_color: &'a ColorAnalysis,
    //     is_light_theme: bool,
    // ) -> &'a ColorAnalysis {
    //     // Find a color that's similar in hue but different in lightness/saturation
    //     colors
    //         .iter()
    //         .filter(|c| {
    //             let hue_diff = (c.hue - normal_color.hue)
    //                 .abs()
    //                 .min(360.0 - (c.hue - normal_color.hue).abs());
    //             hue_diff < 60.0 && // Similar hue
    //             ((is_light_theme && c.lightness > normal_color.lightness) ||
    //              (!is_light_theme && c.lightness < normal_color.lightness) ||
    //              c.saturation < normal_color.saturation)
    //         })
    //         .min_by(|a, b| {
    //             normal_color
    //                 .distance_to(a)
    //                 .partial_cmp(&normal_color.distance_to(b))
    //                 .unwrap_or(std::cmp::Ordering::Equal)
    //         })
    //         .unwrap_or(normal_color)
    // }

    /// Assign colors to semantic roles with better diversity
    #[allow(clippy::too_many_lines)]
    fn assign_semantic_colors<'a>(
        colors: &'a [ColorAnalysis],
        used_colors: &'a mut Vec<&'a ColorAnalysis>,
        normal_color: &'a ColorAnalysis,
        _is_light_theme: bool,
    ) -> SemanticColors<'a> {
        // Create a more diverse color assignment
        let mut available_colors: Vec<_> = colors.iter().collect();

        // Sort by distance from normal color to ensure variety
        available_colors.sort_by(|a, b| {
            normal_color
                .distance_to(b)
                .partial_cmp(&normal_color.distance_to(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ensure all semantic colors are unique and different from each other
        let error_color =
            Self::find_color_by_hue_improved(&available_colors, 0.0, 60.0, normal_color);
        vprtln!(
            V::V,
            "error_color={}",
            Style::new().with_rgb(error_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&error_color.rgb.into()),
                error_color.hue
            ))
        );
        // let mut used_colors = vec![error_color];
        // let mut used_colors = used_colors.to_vec();
        used_colors.push(error_color);

        assert!(used_colors.contains(&error_color));

        let warning_color = Self::find_unique_color_by_hue(
            &available_colors,
            30.0,
            90.0,
            normal_color,
            used_colors,
        );
        vprtln!(
            V::V,
            "warning_color={}",
            Style::new().with_rgb(warning_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&warning_color.rgb.into()),
                warning_color.hue
            ))
        );

        used_colors.push(warning_color);
        assert!(used_colors.contains(&error_color));
        assert!(used_colors.contains(&warning_color));

        let success_color = Self::find_unique_color_by_hue(
            &available_colors,
            90.0,
            150.0,
            normal_color,
            used_colors,
        );
        vprtln!(
            V::V,
            "success_color={}",
            Style::new().with_rgb(success_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&success_color.rgb.into()),
                success_color.hue
            ))
        );

        used_colors.push(success_color);
        assert!(used_colors.contains(&error_color));
        assert!(used_colors.contains(&warning_color));
        assert!(used_colors.contains(&success_color));

        let info_color = Self::find_unique_color_by_hue(
            &available_colors,
            180.0,
            240.0,
            normal_color,
            used_colors,
        );
        vprtln!(
            V::V,
            "info_color={}",
            Style::new().with_rgb(info_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&info_color.rgb.into()),
                info_color.hue
            ))
        );

        used_colors.push(info_color);
        assert!(used_colors.contains(&error_color));
        assert!(used_colors.contains(&warning_color));
        assert!(used_colors.contains(&success_color));
        assert!(used_colors.contains(&info_color));

        let code_color = Self::find_unique_color_by_hue(
            &available_colors,
            240.0,
            300.0,
            normal_color,
            used_colors,
        );
        vprtln!(
            V::V,
            "code_color={}",
            Style::new().with_rgb(code_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&code_color.rgb.into()),
                code_color.hue
            ))
        );

        used_colors.push(code_color);
        assert!(used_colors.contains(&error_color));
        assert!(used_colors.contains(&warning_color));
        assert!(used_colors.contains(&success_color));
        assert!(used_colors.contains(&info_color));
        assert!(used_colors.contains(&code_color));

        let emphasis_color = Self::find_unique_color_by_hue(
            &available_colors,
            15.0,
            45.0,
            normal_color,
            used_colors,
        );
        vprtln!(
            V::V,
            "emphasis_color={}",
            Style::new().with_rgb(emphasis_color.rgb).paint(format!(
                "{}, hue={}",
                rgb_to_hex(&emphasis_color.rgb.into()),
                emphasis_color.hue
            ))
        );

        used_colors.push(emphasis_color);
        assert!(used_colors.contains(&error_color));
        assert!(used_colors.contains(&warning_color));
        assert!(used_colors.contains(&success_color));
        assert!(used_colors.contains(&info_color));
        assert!(used_colors.contains(&code_color));
        assert!(used_colors.contains(&emphasis_color));

        SemanticColors {
            error: error_color,
            warning: warning_color,
            success: success_color,
            info: info_color,
            code: code_color,
            emphasis: emphasis_color,
        }
    }

    /// For the most discreet colors: ind the color most different from all used colors but still not bright.
    fn find_most_different_color<'a>(
        colors: &'a [ColorAnalysis],
        used_colors: &[&ColorAnalysis],
        background: &'a ColorAnalysis,
    ) -> &'a ColorAnalysis {
        vprtln!(V::V, "1. used_colors.len()={}", used_colors.len());
        colors
            .iter()
            .filter(|c| {
                !used_colors.iter().any(|&used| {
                    let lightness_diff = (c.lightness - background.lightness).abs();
                    // vprtln!(V::V,
                    //     "c={}, used={}, distance={}, lightness_diff={lightness_diff}",
                    //     styling::rgb_to_hex(&c.rgb.into()),
                    //     styling::rgb_to_hex(&used.rgb.into()),
                    //     used.distance_to(c)
                    // );
                    used == *c /* || used.distance_to(c) < 20.0 */
                    || lightness_diff > 15.0
                })
            })
            .max_by(|a, b| {
                let min_dist_a = used_colors
                    .iter()
                    .map(|used| used.distance_to(a))
                    .fold(f32::INFINITY, f32::min);
                let min_dist_b = used_colors
                    .iter()
                    .map(|used| used.distance_to(b))
                    .fold(f32::INFINITY, f32::min);
                min_dist_a
                    .partial_cmp(&min_dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&colors[0])
    }

    /// Select heading colors ensuring they're different from all used colors
    fn select_unique_heading_colors<'a>(
        colors: &'a [ColorAnalysis],
        used_colors: &[&'a ColorAnalysis],
    ) -> (&'a ColorAnalysis, &'a ColorAnalysis, &'a ColorAnalysis) {
        let mut available: Vec<_> = colors
            .iter()
            .filter(|c| !used_colors.iter().any(|used| used.distance_to(c) < 25.0))
            .collect();

        // Sort by visual distinctiveness (combination of saturation and contrast)
        available.sort_by(|a, b| {
            // let score_a = a.saturation + (a.lightness - 0.5).abs();
            // let score_b = b.saturation + (b.lightness - 0.5).abs();
            // score_b
            //     .partial_cmp(&score_a)
            //     .unwrap_or(std::cmp::Ordering::Equal)
            b.hue
                .partial_cmp(&a.hue)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut headings = available
            .iter()
            .take(3)
            .copied()
            .collect::<Vec<&ColorAnalysis>>();

        headings.sort_by(|a, b| {
            a.lightness
                .partial_cmp(&b.lightness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let h1 = headings.first().copied().unwrap_or_else(|| {
            vprtln!(
                V::V,
                "Defaulting HD1 to {}",
                Style::new()
                    .with_rgb(colors[0].rgb)
                    .paint(format!("{:?}", &colors[0].rgb))
            );
            &colors[0]
        });
        let h2 = headings.get(1).copied().unwrap_or_else(|| {
            vprtln!(
                V::V,
                "Defaulting HD2 to {}",
                Style::new()
                    .with_rgb(colors[1 % colors.len()].rgb)
                    .paint(format!("{:?}", &colors[1 % colors.len()].rgb))
            );
            &colors[1 % colors.len()]
        });
        let h3 = headings.get(2).copied().unwrap_or_else(|| {
            vprtln!(
                V::V,
                "Defaulting HD3 to {}",
                Style::new()
                    .with_rgb(colors[2 % colors.len()].rgb)
                    .paint(format!("{:?}", &colors[2 % colors.len()].rgb))
            );
            &colors[2 % colors.len()]
        });

        (h1, h2, h3)
    }

    /// Improved hue-based color finding with better fallbacks
    fn find_color_by_hue_improved<'a>(
        colors: &[&'a ColorAnalysis],
        hue_start: f32,
        hue_end: f32,
        fallback: &'a ColorAnalysis,
    ) -> &'a ColorAnalysis {
        // First try: exact hue match
        if let Some(color) = colors
            .iter()
            .filter(|c| {
                let hue = c.hue;
                hue >= hue_start && hue < hue_end
            })
            // .max_by(|a, b| {
            //     a.frequency
            //         .partial_cmp(&b.frequency)
            //         .unwrap_or(std::cmp::Ordering::Equal)
            // })
            .min_by(|a, b| {
                a.hue
                    .partial_cmp(&b.hue)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            return color;
        }

        // Second try: colors with good contrast to fallback
        if let Some(color) = colors
            .iter()
            .filter(|c| c.distance_to(fallback) > 10.0)
            .max_by(|a, b| {
                a.distance_to(fallback)
                    .partial_cmp(&b.distance_to(fallback))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            return color;
        }

        // Final fallback
        colors.first().copied().unwrap_or(fallback)
    }

    /// Find unique color by hue that doesn't conflict with already used colors
    fn find_unique_color_by_hue<'a>(
        colors: &[&'a ColorAnalysis],
        hue_start: f32,
        hue_end: f32,
        fallback: &'a ColorAnalysis,
        used_colors: &[&ColorAnalysis],
    ) -> &'a ColorAnalysis {
        vprtln!(
            V::V,
            "Finding color in hue range {:.0}-{:.0}:",
            hue_start,
            hue_end
        );
        vprtln!(V::V, "Available colors:");
        for (i, color) in colors.iter().enumerate() {
            vprtln!(
                V::V,
                "  [{}] hue={:.0} rgb={} {}",
                i,
                color.hue,
                rgb_to_hex(&color.rgb.into()),
                Style::new().with_rgb(color.rgb).paint("■")
            );
        }
        // First try: exact hue match that's not already used
        if let Some(color) = colors
            .iter()
            .filter(|c| {
                let hue = c.hue;
                hue >= hue_start
                    && hue < hue_end
                    && !used_colors.iter().any(|used| {
                        let distance_to = used.distance_to(c);
                        vprtln!(
                            V::V,
                            "c={}, used={}, distance_to={distance_to}, eligible: {}",
                            Style::new()
                                .with_rgb(c.rgb)
                                .paint(rgb_to_hex(&c.rgb.into())),
                            Style::new()
                                .with_rgb(used.rgb)
                                .paint(rgb_to_hex(&used.rgb.into())),
                            distance_to >= 15.0
                        );
                        used == *c || distance_to < 15.0
                    })
            })
            .inspect(|c| {
                vprtln!(
                    V::V,
                    "{} made the cut on 1st try",
                    rgb_to_hex(&c.rgb.into())
                );
            })
            .max_by(|a, b| {
                a.frequency
                    .partial_cmp(&b.frequency)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            vprtln!(V::V, "1. Returning {}", rgb_to_hex(&color.rgb.into()));
            return color;
        }

        // Second try: prefer colors closer to target hue range, but different enough from used colors
        vprtln!(V::V, "2. used_colors.len()={}", used_colors.len());
        let min_distance = 5.0;
        if let Some(color) = colors
            .iter()
            .filter(|c| !used_colors.contains(c))
            .filter(|c| {
                !used_colors.iter().any(|used| {
                    let distance_to = used.distance_to(c);
                    vprtln!(
                        V::V,
                        "c={}, used={}, distance_to={distance_to}, eligible: {}",
                        rgb_to_hex(&c.rgb.into()),
                        rgb_to_hex(&used.rgb.into()),
                        distance_to >= min_distance
                    );
                    distance_to < min_distance
                })
            })
            .inspect(|c| {
                vprtln!(
                    V::V,
                    "{} made the cut on 2nd try",
                    rgb_to_hex(&c.rgb.into())
                );
            })
            .min_by(|a, b| {
                // Prefer colors closer to the target hue range
                let hue_distance_a = {
                    let hue = a.hue;
                    if hue >= hue_start && hue < hue_end {
                        0.0 // Perfect match
                    } else {
                        // Calculate shortest angular distance to the range
                        let dist_to_start =
                            (hue - hue_start).abs().min(360.0 - (hue - hue_start).abs());
                        let dist_to_end = (hue - hue_end).abs().min(360.0 - (hue - hue_end).abs());
                        dist_to_start.min(dist_to_end)
                    }
                };
                let hue_distance_b = {
                    let hue = b.hue;
                    if hue >= hue_start && hue < hue_end {
                        0.0 // Perfect match
                    } else {
                        let dist_to_start =
                            (hue - hue_start).abs().min(360.0 - (hue - hue_start).abs());
                        let dist_to_end = (hue - hue_end).abs().min(360.0 - (hue - hue_end).abs());
                        dist_to_start.min(dist_to_end)
                    }
                };
                hue_distance_a
                    .partial_cmp(&hue_distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            vprtln!(V::V, "2. Returning {}", rgb_to_hex(&color.rgb.into()));
            return color;
        }

        // Final fallback
        vprtln!(
            V::V,
            "3. Final fallback: Returning {}",
            rgb_to_hex(&colors.first().unwrap_or(&fallback).rgb.into())
        );
        colors.first().copied().unwrap_or(fallback)
    }

    // /// Find the best color within a hue range
    // fn find_color_by_hue<'a>(
    //     &self,
    //     accent_colors: &[&'a ColorAnalysis],
    //     hue_start: f32,
    //     hue_end: f32,
    //     fallback: &'a ColorAnalysis,
    // ) -> &'a ColorAnalysis {
    //     accent_colors
    //         .iter()
    //         .filter(|c| {
    //             let hue = c.hue;
    //             hue >= hue_start && hue < hue_end
    //         })
    //         .max_by(|a, b| {
    //             a.frequency
    //                 .partial_cmp(&b.frequency)
    //                 .unwrap_or(std::cmp::Ordering::Equal)
    //         })
    //         .copied()
    //         .or_else(|| accent_colors.first().copied())
    //         .unwrap_or(fallback)
    // }

    /// Select the best background color ensuring good contrast
    #[allow(clippy::cast_precision_loss)]
    fn select_background_color(colors: &[ColorAnalysis], is_light_theme: bool) -> ColorAnalysis {
        // Force background to match theme type expectation
        if is_light_theme {
            // Light theme should have light background
            let light_candidates: Vec<&ColorAnalysis> = colors
                .iter()
                .filter(|c| c.lightness > 0.8 && c.saturation < 0.2)
                .collect();

            light_candidates.first().map_or_else(
                || ColorAnalysis::new([248, 248, 248], 0.5),
                |bg_color| (*bg_color).clone(),
            )
        } else {
            // Dark theme should have dark background
            let dark_candidates: Vec<&ColorAnalysis> = colors
                .iter()
                .filter(|c| c.lightness < 0.2 && c.saturation < 0.3)
                .collect();

            dark_candidates.first().map_or_else(
                || {
                    // Smart dark background based on dominant colors
                    let avg_hue: f32 = colors
                        .iter()
                        .filter(|c| c.saturation > 0.1)
                        .map(|c| c.hue)
                        .sum::<f32>()
                        / colors.len().max(1) as f32;

                    if (0.0..=60.0).contains(&avg_hue) {
                        // Warm dominant colors - use cool dark background
                        ColorAnalysis::new([20, 25, 30], 0.5) // Cool dark blue-gray
                    } else {
                        ColorAnalysis::new([25, 25, 25], 0.5) // Standard dark gray
                    }
                },
                |bg_color| (*bg_color).clone(),
            )
        }
    }

    /// Generate a theme name from the image path
    fn generate_theme_name<P: AsRef<Path>>(&self, image_path: P) -> String {
        let path = image_path.as_ref();
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("generated");

        self.config.theme_name_prefix.as_ref().map_or_else(
            || format!("image-{base_name}"),
            |prefix| format!("{prefix}-{base_name}"),
        )
    }
}

impl Default for ImageThemeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Helper: RGB -> HSL
#[allow(clippy::many_single_char_names)]
fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
    let r = f32::from(rgb[0]) / 255.0;
    let g = f32::from(rgb[1]) / 255.0;
    let b = f32::from(rgb[2]) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let s;
    let mut h;

    if delta == 0.0 {
        h = 0.0;
        s = 0.0;
    } else {
        s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        let error_margin = 0.001;
        h = if (max - r).abs() < error_margin {
            ((g - b) / delta) % 6.0
        } else if (max - g).abs() < error_margin {
            ((b - r) / delta) + 2.0
        } else {
            ((r - g) / delta) + 4.0
        } * 60.0;

        // Ensure hue is positive
        if h < 0.0 {
            h += 360.0;
        }
    }

    (h, s, l)
}

// Helper: HSL -> RGB
#[allow(
    clippy::cast_possible_truncation,
    clippy::many_single_char_names,
    clippy::cast_sign_loss
)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - 2.0f32.mul_add(l, -1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 | _ => (c, 0.0, x),
    };

    let m = l - c / 2.0;
    let (r, g, b) = (r1 + m, g1 + m, b1 + m);

    [
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    ]
}

/// Convenience function to generate a theme from an image file with default settings
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
pub fn generate_theme_from_image<P: AsRef<Path>>(image_path: P) -> StylingResult<Theme> {
    let generator = ImageThemeGenerator::new();
    generator.generate_from_file(image_path)
}

/// Convenience function to generate a theme from an image file with custom configuration
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
pub fn generate_theme_from_image_with_config<P: AsRef<Path>>(
    image_path: P,
    config: ImageThemeConfig,
) -> StylingResult<Theme> {
    let generator = ImageThemeGenerator::with_config(config);
    generator.generate_from_file(image_path)
}

/// Save a theme directly to a TOML file
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
pub fn save_theme_to_file<P: AsRef<Path>>(theme: &Theme, file_path: P) -> StylingResult<()> {
    let toml_content = theme_to_toml(theme)?;
    std::fs::write(file_path, toml_content).map_err(StylingError::Io)?;
    Ok(())
}

/// Generate a theme from an image and save it directly to a TOML file
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
pub fn generate_and_save_theme<P: AsRef<Path>, Q: AsRef<Path>>(
    image_path: P,
    output_path: Q,
    config: Option<ImageThemeConfig>,
) -> StylingResult<Theme> {
    let config = config.unwrap_or_default();
    let generator = ImageThemeGenerator::with_config(config);
    let theme = generator.generate_from_file(&image_path)?;
    save_theme_to_file(&theme, output_path)?;
    Ok(theme)
}

/// Generate TOML representation of a theme matching the format of built-in themes
///
/// # Errors
///
/// This function will bubble up any i/o errors encountered.
#[allow(clippy::too_many_lines)]
pub fn theme_to_toml(theme: &Theme) -> StylingResult<String> {
    let mut toml = String::new();

    // Header information - match the format of existing themes
    let _ = writeln!(toml, "name = {:?}", theme.name);
    let _ = writeln!(toml, "description = {:?}", theme.description);
    let _ = writeln!(
        toml,
        "term_bg_luma = {:?}",
        format!("{:?}", theme.term_bg_luma).to_lowercase()
    );
    let _ = writeln!(
        toml,
        "min_color_support = {:?}",
        match theme.min_color_support {
            // crate::ColorSupport::TrueColor => "true_color",
            crate::ColorSupport::Color256 => "color256",
            crate::ColorSupport::Basic => "basic",
            _ => "true_color",
        }
    );
    let _ = writeln!(toml, "backgrounds = {:?}", theme.backgrounds);

    // Format bg_rgbs to match existing theme format
    // toml.push_str("bg_rgbs = [[\n");
    let _ = writeln!(toml, "bg_rgbs = [[");
    for rgb in &theme.bg_rgbs {
        let _ = writeln!(toml, "    {},", rgb.0);
        let _ = writeln!(toml, "    {},", rgb.1);
        let _ = writeln!(toml, "    {},", rgb.2);
    }
    // toml.push_str("]]\n\n");
    let _ = writeln!(toml, "]]\n");

    // Palette section - match existing theme format exactly
    let palette_items = [
        ("heading1", &theme.palette.heading1),
        ("heading2", &theme.palette.heading2),
        ("heading3", &theme.palette.heading3),
        ("error", &theme.palette.error),
        ("warning", &theme.palette.warning),
        ("success", &theme.palette.success),
        ("info", &theme.palette.info),
        ("emphasis", &theme.palette.emphasis),
        ("code", &theme.palette.code),
        ("normal", &theme.palette.normal),
        ("subtle", &theme.palette.subtle),
        ("hint", &theme.palette.hint),
        ("debug", &theme.palette.debug),
        ("link", &theme.palette.link),
        ("quote", &theme.palette.quote),
        ("commentary", &theme.palette.commentary),
    ];

    for (role_name, style) in palette_items {
        let _ = writeln!(toml, "[palette.{role_name}]");

        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                crate::ColorValue::TrueColor { rgb } => {
                    toml.push_str("rgb = [\n");
                    let _ = writeln!(toml, "    {},", rgb[0]);
                    let _ = writeln!(toml, "    {},", rgb[1]);
                    let _ = writeln!(toml, "    {},", rgb[2]);
                    toml.push_str("]\n");
                }
                crate::ColorValue::Color256 { color256 } => {
                    let rgb = color_256_to_rgb(*color256);
                    toml.push_str("rgb = [\n");
                    let _ = writeln!(toml, "    {},", rgb[0]);
                    let _ = writeln!(toml, "    {},", rgb[1]);
                    let _ = writeln!(toml, "    {},", rgb[2]);
                    toml.push_str("]\n");
                }
                crate::ColorValue::Basic { .. } => {
                    toml.push_str("rgb = [\n");
                    toml.push_str("    128,\n");
                    toml.push_str("    128,\n");
                    toml.push_str("    128,\n");
                    toml.push_str("]\n");
                }
            }
        }

        // Add style attributes
        let mut style_attrs = Vec::new();
        if style.bold {
            style_attrs.push(r#""bold""#);
        }
        if style.italic {
            style_attrs.push(r#""italic""#);
        }
        if style.dim {
            style_attrs.push(r#""dim""#);
        }
        if style.underline {
            style_attrs.push(r#""underline""#);
        }

        if !style_attrs.is_empty() {
            let _ = writeln!(toml, "style = [{}]", style_attrs.join(", "));
        }

        toml.push('\n');
    }

    Ok(toml)
}

const fn color_256_to_rgb(color: u8) -> [u8; 3] {
    match color {
        0..=15 => {
            let colors = [
                [0, 0, 0],
                [128, 0, 0],
                [0, 128, 0],
                [128, 128, 0],
                [0, 0, 128],
                [128, 0, 128],
                [0, 128, 128],
                [192, 192, 192],
                [128, 128, 128],
                [255, 0, 0],
                [0, 255, 0],
                [255, 255, 0],
                [0, 0, 255],
                [255, 0, 255],
                [0, 255, 255],
                [255, 255, 255],
            ];
            colors[color as usize]
        }
        16..=231 => {
            let n = color - 16;
            let r = (n / 36) * 51;
            let g = ((n % 36) / 6) * 51;
            let b = (n % 6) * 51;
            [r, g, b]
        }
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            [gray, gray, gray]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_contrast_adjustment_light_theme() {
        // Create a light background
        let background = ColorAnalysis::new([240, 240, 240], 0.0); // Very light gray

        // Create a color that's too close to the background
        let low_contrast_color = ColorAnalysis::new([220, 220, 220], 0.0); // Light gray

        // Test that contrast adjustment increases the difference for light theme
        let adjusted = ImageThemeGenerator::adjust_color_contrast(
            &low_contrast_color,
            &background,
            0.6,   // minimum lightness difference
            true,  // is_light_theme
            false, // adjust_saturation
            "TestColor",
        );

        let lightness_diff = (adjusted.lightness - background.lightness).abs();
        assert!(
            lightness_diff >= 0.6,
            "Lightness difference should be at least 0.6, got {}",
            lightness_diff
        );
    }

    #[test]
    fn test_contrast_adjustment_dark_theme() {
        // Create a dark background
        let background = ColorAnalysis::new([30, 30, 30], 0.0); // Very dark gray

        // Create a color that's too close to the background
        let low_contrast_color = ColorAnalysis::new([50, 50, 50], 0.0); // Dark gray

        // Test that contrast adjustment increases the difference for dark theme
        let adjusted = ImageThemeGenerator::adjust_color_contrast(
            &low_contrast_color,
            &background,
            0.7,   // minimum lightness difference (semantic colors)
            false, // is_light_theme
            false, // adjust_saturation
            "TestColor",
        );

        let lightness_diff = (adjusted.lightness - background.lightness).abs();
        assert!(
            lightness_diff >= 0.7,
            "Lightness difference should be at least 0.7, got {}",
            lightness_diff
        );
    }

    #[test]
    fn test_saturation_adjustment() {
        let background = ColorAnalysis::new([128, 128, 128], 0.0); // Mid gray
        let high_saturation_color = ColorAnalysis::new([255, 0, 0], 0.0); // Pure red

        let original_saturation = high_saturation_color.saturation;

        // Test light theme - should boost saturation
        let adjusted_light = ImageThemeGenerator::adjust_color_contrast(
            &high_saturation_color,
            &background,
            0.6,
            true, // is_light_theme
            true, // adjust_saturation
            "TestColor",
        );

        // Should have boosted saturation for light theme
        assert!(
            adjusted_light.saturation > original_saturation,
            "Light theme saturation should be boosted from {} to {}",
            original_saturation,
            adjusted_light.saturation
        );

        // Test dark theme - should reduce saturation
        let adjusted_dark = ImageThemeGenerator::adjust_color_contrast(
            &high_saturation_color,
            &background,
            0.6,
            false, // is_light_theme (dark theme)
            true,  // adjust_saturation
            "TestColor",
        );

        // Should have reduced saturation for dark theme
        assert!(
            adjusted_dark.saturation < original_saturation,
            "Dark theme saturation should be reduced from {} to {}",
            original_saturation,
            adjusted_dark.saturation
        );
        assert!(
            adjusted_dark.saturation >= 0.05,
            "Saturation should not go below minimum of 0.05"
        );
    }

    #[test]
    fn test_contrast_adjustment_preserves_hue() {
        let background = ColorAnalysis::new([128, 128, 128], 0.0);
        let test_color = ColorAnalysis::new([100, 150, 200], 0.0); // Some blue-ish color

        let original_hue = test_color.hue;

        let adjusted = ImageThemeGenerator::adjust_color_contrast(
            &test_color,
            &background,
            0.6,
            true,
            false,
            "TestColor",
        );

        // Hue should remain the same
        assert!(
            (adjusted.hue - original_hue).abs() < 1.0,
            "Hue should be preserved, original: {}, adjusted: {}",
            original_hue,
            adjusted.hue
        );
    }

    fn create_test_image() -> DynamicImage {
        let mut img = RgbImage::new(100, 100);

        // Fill with different colored sections
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let color = match (x / 25, y / 25) {
                (0, 0) => Rgb([255, 0, 0]),   // Red
                (1, 0) => Rgb([0, 255, 0]),   // Green
                (2, 0) => Rgb([0, 0, 255]),   // Blue
                (3, 0) => Rgb([255, 255, 0]), // Yellow
                _ => Rgb([128, 128, 128]),    // Gray
            };
            *pixel = color;
        }

        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn test_color_analysis_creation() {
        let color = ColorAnalysis::new([255, 0, 0], 0.5);
        assert_eq!(color.rgb, [255, 0, 0]);
        assert_eq!(color.frequency, 0.5);
        // Red should have hue around 0 degrees
        assert!((color.hue - 0.0).abs() < 10.0);
    }

    #[test]
    fn test_theme_generation() {
        let generator = ImageThemeGenerator::new();
        let test_image = create_test_image();

        let theme = generator
            .generate_from_image(&test_image, "test-theme".to_string())
            .expect("Should generate theme successfully");

        assert_eq!(theme.name, "test-theme");
        assert!(!theme.bg_rgbs.is_empty());
    }

    #[test]
    fn test_dominant_color_extraction() {
        let generator = ImageThemeGenerator::new();
        let test_image = create_test_image();

        let colors = generator
            .extract_dominant_colors(&test_image)
            .expect("Should extract colors successfully");

        assert!(!colors.is_empty());
        assert!(colors.len() <= generator.config.color_count);

        // Check that frequencies sum to approximately 1.0
        let total_frequency: f32 = colors.iter().map(|(_, freq)| freq).sum();
        assert!((total_frequency - 1.0).abs() < 0.2); // More lenient threshold
    }

    #[test]
    fn test_toml_generation_validity() {
        let generator = ImageThemeGenerator::new();
        let test_image = create_test_image();

        let theme = generator
            .generate_from_image(&test_image, "toml-test".to_string())
            .expect("Should generate theme successfully");

        let toml_content = theme_to_toml(&theme).expect("Should generate TOML successfully");

        // Test that the generated TOML is valid by parsing it back
        let parsed: toml::Value =
            toml::from_str(&toml_content).expect("Generated TOML should be valid and parseable");

        // Verify key sections exist
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("description").is_some());
        assert!(parsed.get("term_bg_luma").is_some());
        assert!(parsed.get("min_color_support").is_some());
        assert!(parsed.get("backgrounds").is_some());
        assert!(parsed.get("bg_rgbs").is_some());
        assert!(parsed.get("palette").is_some());

        // Verify palette has expected roles
        let palette = parsed.get("palette").unwrap().as_table().unwrap();
        assert!(palette.contains_key("normal"));
        assert!(palette.contains_key("error"));
        assert!(palette.contains_key("success"));
        assert!(palette.contains_key("heading1"));

        // Verify color format
        let normal = palette.get("normal").unwrap().as_table().unwrap();
        assert!(normal.contains_key("rgb"));
        let rgb = normal.get("rgb").unwrap().as_array().unwrap();
        assert_eq!(rgb.len(), 3);
    }

    #[test]
    fn test_save_theme_to_file() {
        let generator = ImageThemeGenerator::new();
        let test_image = create_test_image();

        let theme = generator
            .generate_from_image(&test_image, "file-test".to_string())
            .expect("Should generate theme successfully");

        // Save to a temporary file
        let temp_file = "test_generated_theme.toml";
        save_theme_to_file(&theme, temp_file).expect("Should save theme to file");

        // Read back and validate
        let content = std::fs::read_to_string(temp_file).expect("Should read saved file");

        // Validate it's proper TOML
        let parsed: toml::Value =
            toml::from_str(&content).expect("Saved file should be valid TOML");

        // Check structure
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("palette").is_some());

        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_generate_light_theme_toml() {
        let config = ImageThemeConfig {
            force_theme_type: Some(TermBgLuma::Light),
            color_count: 10,
            theme_name_prefix: Some("test".to_string()),
            ..Default::default()
        };

        let generator = ImageThemeGenerator::with_config(config);
        let test_image = create_test_image();

        let theme = generator
            .generate_from_image(&test_image, "light-comparison".to_string())
            .expect("Should generate light theme successfully");

        let toml_content = theme_to_toml(&theme).expect("Should generate TOML successfully");

        // Save for comparison
        let light_file = "test_light_theme.toml";
        std::fs::write(light_file, &toml_content).expect("Should save light theme");

        // Validate it's proper TOML
        let parsed: toml::Value =
            toml::from_str(&toml_content).expect("Generated TOML should be valid and parseable");

        // Verify it's a light theme
        let term_bg_luma = parsed.get("term_bg_luma").unwrap().as_str().unwrap();
        assert_eq!(term_bg_luma, "light");

        // Clean up
        let _ = std::fs::remove_file(light_file);
    }
}
