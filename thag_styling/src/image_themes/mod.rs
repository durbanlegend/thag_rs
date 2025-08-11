//! Image-based theme generation for extracting color palettes from images
//!
//! This module provides functionality to analyze images and generate terminal color themes
//! based on the dominant colors found in the image. It uses color clustering to identify
//! the most prominent colors and intelligently maps them to semantic roles.

#![cfg(feature = "image_themes")]

use crate::{ColorSupport, Palette, Style, StylingError, StylingResult, TermBgLuma, Theme};
use image::{DynamicImage, ImageReader};
use palette::{FromColor, Hsl, IntoColor, Lab, Srgb};
use std::path::{Path, PathBuf};

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
        let srgb = Srgb::new(
            f32::from(rgb[0]) / 255.0,
            f32::from(rgb[1]) / 255.0,
            f32::from(rgb[2]) / 255.0,
        );

        let lab: Lab = srgb.into_color();
        let hsl: Hsl = Hsl::from_color(srgb);

        Self {
            rgb,
            lab,
            hue: hsl.hue.into_positive_degrees(),
            saturation: hsl.saturation,
            lightness: hsl.lightness,
            frequency,
        }
    }

    /// Check if this color is suitable as a background color
    fn is_background_suitable(&self) -> bool {
        // Background colors should be neutral (low saturation) and either very light or very dark
        self.saturation < 0.3 && (self.lightness < 0.2 || self.lightness > 0.8)
    }

    /// Check if this color is suitable for text (good contrast potential)
    fn is_text_suitable(&self, is_light_theme: bool) -> bool {
        if is_light_theme {
            // For light themes, text should be dark
            self.lightness < 0.4
        } else {
            // For dark themes, text should be light
            self.lightness > 0.6
        }
    }

    /// Check if this color is suitable as an accent color
    fn is_accent_suitable(&self, saturation_threshold: f32) -> bool {
        self.saturation >= saturation_threshold
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
        let color_analysis = self.analyze_colors(dominant_colors);

        let is_light_theme = self.determine_theme_type(&color_analysis);
        let palette = self.map_colors_to_roles(&color_analysis, is_light_theme)?;

        let background_rgb = self.select_background_color(&color_analysis, is_light_theme);

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
                background_rgb.0, background_rgb.1, background_rgb.2
            )],
            bg_rgbs: vec![background_rgb],
            palette,
        })
    }

    /// Extract dominant colors from an image using simple k-means-like clustering
    fn extract_dominant_colors(&self, image: &DynamicImage) -> StylingResult<Vec<([u8; 3], f32)>> {
        let rgb_image = image.to_rgb8();
        let pixels: Vec<[u8; 3]> = rgb_image.pixels().map(|p| [p[0], p[1], p[2]]).collect();

        if pixels.is_empty() {
            return Err(StylingError::Generic(
                "Image contains no pixels".to_string(),
            ));
        }

        // Simple color quantization by reducing the color space
        let mut color_counts: std::collections::HashMap<[u8; 3], usize> =
            std::collections::HashMap::new();

        // Quantize colors to reduce noise
        for pixel in &pixels {
            let quantized = [
                (pixel[0] / 16) * 16, // Reduce to 16 levels per channel
                (pixel[1] / 16) * 16,
                (pixel[2] / 16) * 16,
            ];
            *color_counts.entry(quantized).or_insert(0) += 1;
        }

        // Sort colors by frequency and take the most common ones
        let mut colors_by_frequency: Vec<_> = color_counts.into_iter().collect();
        colors_by_frequency.sort_by(|a, b| b.1.cmp(&a.1));

        let total_pixels = pixels.len() as f32;
        let mut result = Vec::new();

        for (color, count) in colors_by_frequency
            .into_iter()
            .take(self.config.color_count)
        {
            let frequency = count as f32 / total_pixels;
            result.push((color, frequency));
        }

        // If we don't have enough colors, add some default ones
        while result.len() < 8 {
            let gray_level = (result.len() * 32) as u8;
            result.push(([gray_level, gray_level, gray_level], 0.01));
        }

        Ok(result)
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
        if let Some(forced_type) = &self.config.force_theme_type {
            return *forced_type == TermBgLuma::Light;
        }

        if !self.config.auto_detect_theme_type {
            // Default to dark theme if not auto-detecting
            return false;
        }

        // Calculate weighted average lightness of all colors
        let total_weight: f32 = colors.iter().map(|c| c.frequency).sum();
        if total_weight == 0.0 {
            return false;
        }

        let weighted_lightness: f32 = colors
            .iter()
            .map(|c| c.lightness * c.frequency)
            .sum::<f32>()
            / total_weight;

        // Theme is light if average lightness is above threshold
        weighted_lightness > self.config.light_threshold
    }

    /// Map extracted colors to semantic roles
    fn map_colors_to_roles(
        &self,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> StylingResult<Palette> {
        // Find suitable colors for different categories
        let text_colors: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_text_suitable(is_light_theme))
            .collect();

        let accent_colors: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_accent_suitable(self.config.saturation_threshold))
            .collect();

        // Select normal text color (highest frequency suitable text color)
        let normal_color = if let Some(best_text) = text_colors.iter().max_by(|a, b| {
            a.frequency
                .partial_cmp(&b.frequency)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            *best_text
        } else if let Some(first_color) = colors.first() {
            first_color
        } else {
            return Err(StylingError::Generic(
                "No suitable colors found".to_string(),
            ));
        };

        // Select subtle color (lower contrast version of normal)
        let subtle_color = if let Some(best_subtle) = text_colors
            .iter()
            .filter(|c| {
                c.lightness > normal_color.lightness || c.saturation < normal_color.saturation
            })
            .min_by(|a, b| {
                normal_color
                    .distance_to(a)
                    .partial_cmp(&normal_color.distance_to(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
            *best_subtle
        } else {
            normal_color
        };

        // Map accent colors to semantic roles by hue
        let error_color = self.find_color_by_hue(&accent_colors, 0.0, 60.0, normal_color); // Red range
        let warning_color = self.find_color_by_hue(&accent_colors, 30.0, 90.0, normal_color); // Orange/Yellow range
        let success_color = self.find_color_by_hue(&accent_colors, 90.0, 150.0, normal_color); // Green range
        let info_color = self.find_color_by_hue(&accent_colors, 180.0, 240.0, normal_color); // Cyan/Blue range
        let code_color = self.find_color_by_hue(&accent_colors, 240.0, 300.0, normal_color); // Blue/Purple range
        let emphasis_color = self.find_color_by_hue(&accent_colors, 300.0, 360.0, normal_color); // Purple/Magenta range

        // Heading colors - use accent colors with bold styling
        let heading1_color = accent_colors.first().copied().unwrap_or(normal_color);
        let heading2_color = accent_colors.get(1).copied().unwrap_or(normal_color);
        let heading3_color = accent_colors.get(2).copied().unwrap_or(normal_color);

        Ok(Palette {
            normal: Style::new().with_rgb(normal_color.rgb),
            subtle: Style::new().with_rgb(subtle_color.rgb),
            hint: Style::new().with_rgb(subtle_color.rgb).italic(),
            heading1: Style::new().with_rgb(heading1_color.rgb).bold(),
            heading2: Style::new().with_rgb(heading2_color.rgb).bold(),
            heading3: Style::new().with_rgb(heading3_color.rgb).bold(),
            error: Style::new().with_rgb(error_color.rgb),
            warning: Style::new().with_rgb(warning_color.rgb),
            success: Style::new().with_rgb(success_color.rgb),
            info: Style::new().with_rgb(info_color.rgb),
            code: Style::new().with_rgb(code_color.rgb),
            emphasis: Style::new().with_rgb(emphasis_color.rgb),
            debug: Style::new().with_rgb(subtle_color.rgb).dim(),
            trace: Style::new().with_rgb(subtle_color.rgb).italic().dim(),
        })
    }

    /// Find the best color within a hue range
    fn find_color_by_hue<'a>(
        &self,
        accent_colors: &[&'a ColorAnalysis],
        hue_start: f32,
        hue_end: f32,
        fallback: &'a ColorAnalysis,
    ) -> &'a ColorAnalysis {
        accent_colors
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
            .copied()
            .or_else(|| accent_colors.first().copied())
            .unwrap_or(fallback)
    }

    /// Select the best background color
    fn select_background_color(
        &self,
        colors: &[ColorAnalysis],
        is_light_theme: bool,
    ) -> (u8, u8, u8) {
        let background_candidates: Vec<&ColorAnalysis> = colors
            .iter()
            .filter(|c| c.is_background_suitable())
            .collect();

        if let Some(bg_color) = background_candidates.first() {
            (bg_color.rgb[0], bg_color.rgb[1], bg_color.rgb[2])
        } else {
            // Fallback to default background based on theme type
            if is_light_theme {
                (248, 248, 248) // Light gray
            } else {
                (32, 32, 32) // Dark gray
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
