//! Image-based theme generation for extracting color palettes from images
//!
//! This module provides functionality to analyze images and generate terminal color themes
//! based on the dominant colors found in the image. It uses color clustering to identify
//! the most prominent colors and intelligently maps them to semantic roles.

#![cfg(feature = "image_themes")]

use crate::{
    cprtln, ColorSupport, Palette, Role, Style, StylingError, StylingResult, TermBgLuma, Theme,
};
use image::{DynamicImage, ImageReader};
use palette::{FromColor, Hsl, IntoColor, Lab, Srgb};
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
#[derive(Debug, Clone)]
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
    //     eprintln!(
    //         "self.rgb={},{},{}, is_background_suitable={is_background_suitable}",
    //         self.rgb[0], self.rgb[1], self.rgb[2]
    //     );
    //     is_background_suitable
    // }

    /// Check if this color is suitable for text (good contrast potential)
    fn is_text_suitable(&self, is_light_theme: bool) -> bool {
        let is_text_suitable = if is_light_theme {
            // For light themes, text should be dark but not too dark (avoid pure black)
            self.lightness > 0.15 && self.lightness < 0.6
        } else {
            // For dark themes, text should be light but not too light (avoid pure white)
            self.lightness > 0.6 && self.lightness < 0.95
        };
        eprintln!(
            "is_light_theme={is_light_theme}, self.rgb={}, self.lightness={}, is_text_suitable={is_text_suitable}",
            Style::new().with_rgb(self.rgb).paint(format!("{:?}", self.rgb)), self.lightness
        );
        // dbg!(is_text_suitable);
        is_text_suitable
    }

    /// Check contrast against background
    fn has_good_contrast_against(&self, background: &ColorAnalysis) -> bool {
        let lightness_diff = (self.lightness - background.lightness).abs();
        // dbg!(lightness_diff);
        let has_good_contrast_against = lightness_diff > 0.5; // Minimum contrast requirement
                                                              // dbg!(has_good_contrast_against);
        eprintln!("self.rgb={}, background.lightness={}, lightness_diff={lightness_diff}, has_good_contrast_against={has_good_contrast_against}", Style::new().with_rgb(self.rgb).paint(format!("{:?}", self.rgb)), background.lightness);
        has_good_contrast_against
    }

    /// Check if this color is suitable as an accent color
    fn is_accent_suitable(&self, saturation_threshold: f32) -> bool {
        // dbg!(self.saturation);
        // dbg!(self.lightness);
        // dbg!(
        //     self.saturation >= saturation_threshold && self.lightness > 0.2 && self.lightness < 0.9
        // );

        let is_accent_suitable =
            self.saturation >= saturation_threshold && self.lightness > 0.2 && self.lightness < 0.9;
        // dbg!(is_accent_suitable);
        eprintln!(
            "self.rgb={}, is_accent_suitable={is_accent_suitable}",
            Style::new()
                .with_rgb(self.rgb)
                .paint(format!("{:?}", self.rgb))
        );
        is_accent_suitable
    }

    /// Calculate perceptual distance to another color using Delta E
    fn distance_to(&self, other: &ColorAnalysis) -> f32 {
        let delta_l = self.lab.l - other.lab.l;
        let delta_a = self.lab.a - other.lab.a;
        let delta_b = self.lab.b - other.lab.b;

        // Simplified Delta E calculation
        (delta_l * delta_l + delta_a * delta_a + delta_b * delta_b).sqrt()
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
    pub fn new() -> Self {
        Self {
            config: ImageThemeConfig::default(),
        }
    }

    /// Create a new image theme generator with custom configuration
    pub fn with_config(config: ImageThemeConfig) -> Self {
        Self { config }
    }

    /// Generate a theme from an image file
    pub fn generate_from_file<P: AsRef<Path>>(&self, image_path: P) -> StylingResult<Theme> {
        let image = ImageReader::open(&image_path)
            .map_err(|e| StylingError::Generic(format!("Failed to open image: {}", e)))?
            .decode()
            .map_err(|e| StylingError::Generic(format!("Failed to decode image: {}", e)))?;

        let theme_name = self.generate_theme_name(&image_path);
        self.generate_from_image(image, theme_name)
    }

    /// Generate a theme from a loaded image
    pub fn generate_from_image(
        &self,
        image: DynamicImage,
        theme_name: String,
    ) -> StylingResult<Theme> {
        let dominant_colors = self.extract_dominant_colors(&image)?;
        cprtln!(Role::HD1, "Dominant colors:");
        for (color, freq) in &dominant_colors {
            let (__lab, hsl) = to_lab_hsl(*color);
            eprintln!(
                "{} with frequency {freq}",
                Style::new().with_rgb(*color).paint(format!(
                    "{color:?} = hue: {:.0}",
                    hsl.hue.into_positive_degrees()
                ))
            );
        }

        let color_analysis = self.analyze_colors(dominant_colors);

        let is_light_theme = self.determine_theme_type(&color_analysis);
        let background_color = self.select_background_color(&color_analysis, is_light_theme);
        eprintln!(
            "Selected background color={}",
            Style::new()
                .with_rgb(background_color.rgb)
                .paint(format!("{:?}", background_color.rgb))
        );

        let palette =
            self.map_colors_to_roles(&background_color, &color_analysis, is_light_theme)?;

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
        for (color, count) in colors_by_frequency.into_iter() {
            let frequency = count as f32 / total_pixels;

            // Check if this color is sufficiently different from already selected colors
            let is_diverse = result.is_empty()
                || result.iter().all(|(existing_color, _)| {
                    self.color_distance_euclidean(*existing_color, color) > 60.0
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
        self.ensure_color_diversity(&mut result, total_pixels);

        Ok(result)
    }

    /// Calculate Euclidean distance between two RGB colors
    fn color_distance_euclidean(&self, color1: [u8; 3], color2: [u8; 3]) -> f32 {
        let dr = f32::from(color1[0]) - f32::from(color2[0]);
        let dg = f32::from(color1[1]) - f32::from(color2[1]);
        let db = f32::from(color1[2]) - f32::from(color2[2]);
        (dr * dr + dg * dg + db * db).sqrt()
    }

    /// Ensure extracted colors have good diversity by adding contrasting colors if needed
    fn ensure_color_diversity(&self, colors: &mut Vec<([u8; 3], f32)>, _total_pixels: f32) {
        let min_colors = 8;

        if colors.len() < min_colors {
            // Add strategically chosen contrasting colors
            let fallback_colors = [
                [240, 240, 240], // Light gray
                [60, 60, 60],    // Dark gray
                [200, 80, 80],   // Red
                [80, 200, 80],   // Green
                [80, 80, 200],   // Blue
                [200, 200, 80],  // Yellow
                [200, 80, 200],  // Magenta
                [80, 200, 200],  // Cyan
            ];

            for &fallback in &fallback_colors {
                if colors.len() >= min_colors {
                    break;
                }

                // Only add if sufficiently different from existing colors
                let is_different = colors
                    .iter()
                    .all(|(existing, _)| self.color_distance_euclidean(*existing, fallback) > 80.0);

                if is_different {
                    colors.push((fallback, 0.01)); // Low frequency for fallback colors
                }
            }
        }
    }

    /// Analyze colors and create ColorAnalysis structures
    fn analyze_colors(&self, colors: Vec<([u8; 3], f32)>) -> Vec<ColorAnalysis> {
        colors
            .into_iter()
            .map(|(rgb, freq)| ColorAnalysis::new(rgb, freq))
            .collect()
    }

    /// Determine if the theme should be light or dark
    fn determine_theme_type(&self, colors: &[ColorAnalysis]) -> bool {
        dbg!(self.config.force_theme_type);
        if let Some(forced_type) = &self.config.force_theme_type {
            return *forced_type == TermBgLuma::Light;
        }

        dbg!(self.config.auto_detect_theme_type);

        if !self.config.auto_detect_theme_type {
            // Default to dark theme if not auto-detecting
            return false;
        }

        // Calculate weighted average lightness of all colors
        let total_weight: f32 = colors.iter().map(|c| c.frequency).sum();
        dbg!(total_weight);
        if total_weight == 0.0 {
            return false;
        }

        let weighted_lightness: f32 = colors
            .iter()
            .map(|c| c.lightness * c.frequency)
            .sum::<f32>()
            / total_weight;

        dbg!(weighted_lightness);
        dbg!(self.config.light_threshold);

        // Theme is light if average lightness is above threshold
        weighted_lightness > self.config.light_threshold
    }

    /// Map extracted colors to semantic roles with improved contrast and diversity
    fn map_colors_to_roles(
        &self,
        background_color: &ColorAnalysis,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> StylingResult<Palette> {
        // Find suitable colors for different categories with better filtering
        let text_colors: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_text_suitable(is_light_theme))
            .collect();

        let _accent_colors: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_accent_suitable(self.config.saturation_threshold))
            .collect();

        // If we don't have enough diverse colors, create synthetic ones
        let enhanced_colors = self.enhance_color_palette(colors, is_light_theme);

        //         // Select normal text color ensuring proper contrast with background
        // x        let background_color = enhanced_colors
        //             .iter()
        //             .find(|c| c.is_background_suitable())
        //             .or_else(|| {
        //                 eprintln!("No suitable bg colours found, trying enhanced_colors.first()");
        //                 enhanced_colors.first()
        //             })
        //             .unwrap_or_else(|| {
        //                 eprintln!("No suitable bg colours found, trying enhanced_colors[0]");
        //                 &enhanced_colors[0]
        //             });
        //         eprintln!("background_color={:?}", background_color.rgb);

        let normal_color = if let Some(best_text) = self.select_best_text_color(
            &text_colors,
            &enhanced_colors,
            is_light_theme,
            Some(background_color),
        ) {
            // Ensure the selected text color is actually different from background
            if best_text.distance_to(background_color) < 50.0 {
                eprintln!("Distance < 50, calling ensure_text_contrast");
                self.ensure_text_contrast(&enhanced_colors, background_color, is_light_theme)
            } else {
                eprintln!("Going with best_text");
                best_text
            }
        } else {
            // Ensure we have a proper text color with good contrast
            eprintln!("Calling ensure_text_contrast");
            self.ensure_text_contrast(&enhanced_colors, background_color, is_light_theme)
        };
        dbg!(normal_color);

        // Create a comprehensive unique color assignment
        let mut used_colors = vec![normal_color];

        let subtle_color = self.find_most_different_color(&enhanced_colors, &used_colors);
        used_colors.push(subtle_color);

        let hint_color = self.find_most_different_color(&enhanced_colors, &used_colors);
        used_colors.push(hint_color);

        // Map colors to semantic roles with better contrast and diversity
        let semantic_colors =
            self.assign_semantic_colors(&enhanced_colors, normal_color, is_light_theme);

        // Ensure semantic colors are also unique from normal/subtle
        used_colors.extend(&[
            semantic_colors.error,
            semantic_colors.warning,
            semantic_colors.success,
            semantic_colors.info,
            semantic_colors.code,
            semantic_colors.emphasis,
        ]);

        // Select heading colors with good contrast and uniqueness
        let heading_colors = self.select_unique_heading_colors(&enhanced_colors, &used_colors);

        // Debug and trace should be different from subtle and hint
        let debug_color = self.find_most_different_color(&enhanced_colors, &used_colors);
        let trace_color = if debug_color.distance_to(hint_color) > 30.0 {
            hint_color
        } else {
            self.find_most_different_color(&enhanced_colors, &[debug_color])
        };

        Ok(Palette {
            normal: Style::new().with_rgb(normal_color.rgb),
            subtle: Style::new().with_rgb(subtle_color.rgb),
            hint: Style::new().with_rgb(hint_color.rgb).italic(),
            heading1: Style::new().with_rgb(heading_colors.0.rgb).bold(),
            heading2: Style::new().with_rgb(heading_colors.1.rgb).bold(),
            heading3: Style::new().with_rgb(heading_colors.2.rgb).bold(),
            error: Style::new().with_rgb(semantic_colors.error.rgb),
            warning: Style::new().with_rgb(semantic_colors.warning.rgb),
            success: Style::new().with_rgb(semantic_colors.success.rgb),
            info: Style::new().with_rgb(semantic_colors.info.rgb),
            code: Style::new().with_rgb(semantic_colors.code.rgb),
            emphasis: Style::new().with_rgb(semantic_colors.emphasis.rgb),
            debug: Style::new().with_rgb(debug_color.rgb).dim(),
            trace: Style::new().with_rgb(trace_color.rgb).italic().dim(),
        })
    }

    /// Enhance color palette with synthetic colors if diversity is lacking
    fn enhance_color_palette(
        &self,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> Vec<ColorAnalysis> {
        let mut enhanced = colors.to_vec();

        // Check if we need more diversity
        if self.needs_color_enhancement(&enhanced, is_light_theme) {
            let synthetic_colors = self.generate_synthetic_colors(is_light_theme);
            for synthetic in synthetic_colors {
                // Only add if different enough from existing colors
                if enhanced
                    .iter()
                    .all(|existing| existing.distance_to(&synthetic) > 50.0)
                {
                    enhanced.push(synthetic);
                }
            }
        }

        enhanced
    }

    /// Check if the color palette needs enhancement
    fn needs_color_enhancement(&self, colors: &[ColorAnalysis], _is_light_theme: bool) -> bool {
        // Check for lack of contrast
        let avg_lightness: f32 =
            colors.iter().map(|c| c.lightness).sum::<f32>() / colors.len() as f32;
        let lightness_variance: f32 = colors
            .iter()
            .map(|c| (c.lightness - avg_lightness).powi(2))
            .sum::<f32>()
            / colors.len() as f32;

        // Need enhancement if colors are too similar in lightness or too few colors
        lightness_variance < 0.1 || colors.len() < 6
    }

    /// Generate synthetic colors to improve palette diversity
    fn generate_synthetic_colors(&self, is_light_theme: bool) -> Vec<ColorAnalysis> {
        let base_colors = if is_light_theme {
            vec![
                [180, 30, 30],  // Red
                [30, 150, 30],  // Green
                [30, 30, 180],  // Blue
                [150, 120, 30], // Orange
                [120, 30, 150], // Purple
                [30, 120, 150], // Teal
                [60, 60, 60],   // Dark gray
            ]
        } else {
            vec![
                [220, 80, 80],   // Light red
                [80, 220, 80],   // Light green
                [80, 80, 220],   // Light blue
                [220, 180, 80],  // Yellow
                [180, 80, 220],  // Light purple
                [80, 180, 220],  // Light cyan
                [200, 200, 200], // Light gray
            ]
        };

        base_colors
            .into_iter()
            .map(|rgb| ColorAnalysis::new(rgb, 0.05)) // Low frequency for synthetic colors
            .collect()
    }

    /// Select the best text color with contrast consideration
    fn select_best_text_color<'a>(
        &self,
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
                eprintln!(
                    "Found best text color {}",
                    Style::new()
                        .with_rgb(best.rgb)
                        .paint(format!("{:?}", best.rgb))
                );
                return Some(*best);
            }
        }

        // Fallback to any suitable text color
        let best_text = if let Some(best) = text_colors.iter().max_by(|a, b| {
            a.frequency
                .partial_cmp(&b.frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            eprintln!(
                "Falling back to text color {}",
                Style::new()
                    .with_rgb(best.rgb)
                    .paint(format!("{:?}", best.rgb))
            );
            Some(*best)
        } else {
            // Final fallback: find any color with good contrast
            eprintln!("Falling back to any color with good contrast");
            all_colors
                .iter()
                .find(|c| c.is_text_suitable(is_light_theme))
        };
        eprintln!(
            "Selected best_text {}",
            Style::new()
                .with_rgb(best_text.unwrap().rgb)
                .paint(format!("{:?}", best_text.unwrap().rgb))
        );
        best_text
    }

    /// Ensure we have a text color with proper contrast
    fn ensure_text_contrast<'a>(
        &self,
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
            eprintln!("Found good contrast: {:?}", good_contrast.rgb);
            return good_contrast;
        }

        eprintln!("Falling back to synthetic text color");

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
    fn assign_semantic_colors<'a>(
        &self,
        colors: &'a [ColorAnalysis],
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
            self.find_color_by_hue_improved(&available_colors, 0.0, 60.0, normal_color);
        let mut used_colors = vec![error_color];

        let warning_color = self.find_unique_color_by_hue(
            &available_colors,
            30.0,
            90.0,
            normal_color,
            &used_colors,
        );
        used_colors.push(warning_color);

        let success_color = self.find_unique_color_by_hue(
            &available_colors,
            90.0,
            150.0,
            normal_color,
            &used_colors,
        );
        used_colors.push(success_color);

        let info_color = self.find_unique_color_by_hue(
            &available_colors,
            180.0,
            240.0,
            normal_color,
            &used_colors,
        );
        used_colors.push(info_color);

        let code_color = self.find_unique_color_by_hue(
            &available_colors,
            240.0,
            300.0,
            normal_color,
            &used_colors,
        );
        used_colors.push(code_color);

        let emphasis_color = self.find_unique_color_by_hue(
            &available_colors,
            300.0,
            360.0,
            normal_color,
            &used_colors,
        );

        SemanticColors {
            error: error_color,
            warning: warning_color,
            success: success_color,
            info: info_color,
            code: code_color,
            emphasis: emphasis_color,
        }
    }

    /// Find the color most different from all used colors
    fn find_most_different_color<'a>(
        &self,
        colors: &'a [ColorAnalysis],
        used_colors: &[&ColorAnalysis],
    ) -> &'a ColorAnalysis {
        colors
            .iter()
            .filter(|c| !used_colors.iter().any(|used| used.distance_to(c) < 20.0))
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
        &self,
        colors: &'a [ColorAnalysis],
        used_colors: &[&'a ColorAnalysis],
    ) -> (&'a ColorAnalysis, &'a ColorAnalysis, &'a ColorAnalysis) {
        let mut available: Vec<_> = colors
            .iter()
            .filter(|c| !used_colors.iter().any(|used| used.distance_to(c) < 25.0))
            .collect();

        // Sort by visual distinctiveness (combination of saturation and contrast)
        available.sort_by(|a, b| {
            let score_a = a.saturation + (a.lightness - 0.5).abs();
            let score_b = b.saturation + (b.lightness - 0.5).abs();
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let h1 = available.get(0).copied().unwrap_or_else(|| {
            eprintln!(
                "Defaulting HD1 to {}",
                Style::new()
                    .with_rgb(colors[0].rgb)
                    .paint(format!("{:?}", &colors[0].rgb))
            );
            &colors[0]
        });
        let h2 = available.get(1).copied().unwrap_or_else(|| {
            eprintln!(
                "Defaulting HD2 to {}",
                Style::new()
                    .with_rgb(colors[1 % colors.len()].rgb)
                    .paint(format!("{:?}", &colors[1 % colors.len()].rgb))
            );
            &colors[1 % colors.len()]
        });
        let h3 = available.get(2).copied().unwrap_or_else(|| {
            eprintln!(
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
        &self,
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
            .max_by(|a, b| {
                a.frequency
                    .partial_cmp(&b.frequency)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            return color;
        }

        // Second try: colors with good contrast to fallback
        if let Some(color) = colors
            .iter()
            .filter(|c| c.distance_to(fallback) > 40.0)
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
        &self,
        colors: &[&'a ColorAnalysis],
        hue_start: f32,
        hue_end: f32,
        fallback: &'a ColorAnalysis,
        used_colors: &[&ColorAnalysis],
    ) -> &'a ColorAnalysis {
        // First try: exact hue match that's not already used
        if let Some(color) = colors
            .iter()
            .filter(|c| {
                let hue = c.hue;
                hue >= hue_start
                    && hue < hue_end
                    && !used_colors.iter().any(|used| used.distance_to(c) < 30.0)
            })
            .max_by(|a, b| {
                a.frequency
                    .partial_cmp(&b.frequency)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            return color;
        }

        // Second try: any color that's different enough from used colors
        if let Some(color) = colors
            .iter()
            .filter(|c| !used_colors.iter().any(|used| used.distance_to(c) < 40.0))
            .max_by(|a, b| {
                // Prefer colors that are maximally different from all used colors
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
        {
            return color;
        }

        // Final fallback
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
    fn select_background_color(
        &self,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> ColorAnalysis {
        // Force background to match theme type expectation
        if is_light_theme {
            // Light theme should have light background
            let light_candidates: Vec<&ColorAnalysis> = colors
                .iter()
                .filter(|c| c.lightness > 0.8 && c.saturation < 0.2)
                .collect();

            if let Some(bg_color) = light_candidates.first() {
                (*bg_color).clone()
            } else {
                ColorAnalysis::new([248, 248, 248], 0.5) // Light gray background
            }
        } else {
            // Dark theme should have dark background
            let dark_candidates: Vec<&ColorAnalysis> = colors
                .iter()
                .filter(|c| c.lightness < 0.2 && c.saturation < 0.3)
                .collect();

            if let Some(bg_color) = dark_candidates.first() {
                (*bg_color).clone()
            } else {
                // Smart dark background based on dominant colors
                let avg_hue: f32 = colors
                    .iter()
                    .filter(|c| c.saturation > 0.1)
                    .map(|c| c.hue)
                    .sum::<f32>()
                    / colors.len().max(1) as f32;

                if avg_hue >= 0.0 && avg_hue <= 60.0 {
                    // Warm dominant colors - use cool dark background
                    ColorAnalysis::new([20, 25, 30], 0.5) // Cool dark blue-gray
                } else {
                    ColorAnalysis::new([25, 25, 25], 0.5) // Standard dark gray
                }
            }
        }
    }

    /// Generate a theme name from the image path
    fn generate_theme_name<P: AsRef<Path>>(&self, image_path: P) -> String {
        let path = image_path.as_ref();
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("generated");

        match &self.config.theme_name_prefix {
            Some(prefix) => format!("{}-{}", prefix, base_name),
            None => format!("image-{}", base_name),
        }
    }
}

impl Default for ImageThemeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to generate a theme from an image file with default settings
pub fn generate_theme_from_image<P: AsRef<Path>>(image_path: P) -> StylingResult<Theme> {
    let generator = ImageThemeGenerator::new();
    generator.generate_from_file(image_path)
}

/// Convenience function to generate a theme from an image file with custom configuration
pub fn generate_theme_from_image_with_config<P: AsRef<Path>>(
    image_path: P,
    config: ImageThemeConfig,
) -> StylingResult<Theme> {
    let generator = ImageThemeGenerator::with_config(config);
    generator.generate_from_file(image_path)
}

/// Save a theme directly to a TOML file
pub fn save_theme_to_file<P: AsRef<Path>>(theme: &Theme, file_path: P) -> StylingResult<()> {
    let toml_content = theme_to_toml(theme)?;
    std::fs::write(file_path, toml_content).map_err(|e| StylingError::Io(e))?;
    Ok(())
}

/// Generate a theme from an image and save it directly to a TOML file
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
pub fn theme_to_toml(theme: &Theme) -> StylingResult<String> {
    let mut toml = String::new();

    // Header information - match the format of existing themes
    toml.push_str(&format!("name = {:?}\n", theme.name));
    toml.push_str(&format!("description = {:?}\n", theme.description));
    toml.push_str(&format!(
        "term_bg_luma = {:?}\n",
        format!("{:?}", theme.term_bg_luma).to_lowercase()
    ));
    toml.push_str(&format!(
        "min_color_support = {:?}\n",
        match theme.min_color_support {
            crate::ColorSupport::TrueColor => "true_color",
            crate::ColorSupport::Color256 => "color256",
            crate::ColorSupport::Basic => "basic",
            _ => "true_color",
        }
    ));
    toml.push_str(&format!("backgrounds = {:?}\n", theme.backgrounds));

    // Format bg_rgbs to match existing theme format
    toml.push_str("bg_rgbs = [[\n");
    for rgb in &theme.bg_rgbs {
        toml.push_str(&format!("    {},\n", rgb.0));
        toml.push_str(&format!("    {},\n", rgb.1));
        toml.push_str(&format!("    {},\n", rgb.2));
    }
    toml.push_str("]]\n\n");

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
        ("trace", &theme.palette.trace),
    ];

    for (role_name, style) in palette_items {
        toml.push_str(&format!("[palette.{}]\n", role_name));

        if let Some(color_info) = &style.foreground {
            match &color_info.value {
                crate::ColorValue::TrueColor { rgb } => {
                    toml.push_str("rgb = [\n");
                    toml.push_str(&format!("    {},\n", rgb[0]));
                    toml.push_str(&format!("    {},\n", rgb[1]));
                    toml.push_str(&format!("    {},\n", rgb[2]));
                    toml.push_str("]\n");
                }
                crate::ColorValue::Color256 { color256 } => {
                    let rgb = color_256_to_rgb(*color256);
                    toml.push_str("rgb = [\n");
                    toml.push_str(&format!("    {},\n", rgb[0]));
                    toml.push_str(&format!("    {},\n", rgb[1]));
                    toml.push_str(&format!("    {},\n", rgb[2]));
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
            style_attrs.push("\"bold\"");
        }
        if style.italic {
            style_attrs.push("\"italic\"");
        }
        if style.dim {
            style_attrs.push("\"dim\"");
        }
        if style.underline {
            style_attrs.push("\"underline\"");
        }

        if !style_attrs.is_empty() {
            toml.push_str(&format!("style = [{}]\n", style_attrs.join(", ")));
        }

        toml.push('\n');
    }

    Ok(toml)
}

fn color_256_to_rgb(color: u8) -> [u8; 3] {
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
            .generate_from_image(test_image, "test-theme".to_string())
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
            .generate_from_image(test_image, "toml-test".to_string())
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
            .generate_from_image(test_image, "file-test".to_string())
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
            .generate_from_image(test_image, "light-comparison".to_string())
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
