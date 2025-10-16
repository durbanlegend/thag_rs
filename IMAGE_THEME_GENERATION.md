# Image Theme Generation

This document describes the new image-based theme generation feature in `thag_styling`, which allows you to create terminal color themes by analyzing images and extracting their dominant colors.

## Overview

The image theme generation system uses color clustering algorithms to identify the most prominent colors in an image and intelligently maps them to semantic roles (like error, success, headings, etc.) based on color theory principles.

## Features

- **Automatic color extraction** from images using color quantization
- **Intelligent color mapping** to semantic roles based on hue ranges
- **Automatic theme type detection** (light/dark) based on image brightness
- **Customizable generation parameters** for fine-tuning results
- **TOML export** compatible with thag's existing theme system
- **Support for multiple image formats** (PNG, JPEG, GIF, BMP, TIFF, WebP)

## Usage

### Basic Example

```rust
use thag_styling::{ImageThemeGenerator, ImageThemeConfig};

// Generate a theme from an image file
let generator = ImageThemeGenerator::new();
let theme = generator.generate_from_file("sunset.jpg")?;

println!("Generated theme: {}", theme.name);
println!("Theme type: {:?}", theme.term_bg_luma);
```

### Advanced Configuration

```rust
use thag_styling::{ImageThemeConfig, ImageThemeGenerator, TermBgLuma};

let config = ImageThemeConfig {
    color_count: 20,                              // Extract 20 dominant colors
    light_threshold: 0.7,                         // Brightness threshold for light themes
    saturation_threshold: 0.4,                    // Minimum saturation for accent colors
    force_theme_type: Some(TermBgLuma::Dark),     // Force dark theme
    theme_name_prefix: Some("custom".to_string()), // Custom name prefix
    ..Default::default()
};

let generator = ImageThemeGenerator::with_config(config);
let theme = generator.generate_from_image(image, "my-theme".to_string())?;
```

### Running the Example

```bash
cd thag_rs/thag_styling
cargo run --example image_theme_generation --features image_themes
```

## Configuration Options

The `ImageThemeConfig` struct provides several customization options:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `color_count` | `usize` | 16 | Number of dominant colors to extract |
| `light_threshold` | `f32` | 0.7 | Brightness threshold for determining light themes (0.0-1.0) |
| `saturation_threshold` | `f32` | 0.3 | Minimum saturation for accent colors (0.0-1.0) |
| `auto_detect_theme_type` | `bool` | `true` | Whether to automatically detect light/dark theme |
| `force_theme_type` | `Option<TermBgLuma>` | `None` | Force theme to be light or dark |
| `theme_name_prefix` | `Option<String>` | `None` | Custom prefix for theme names |

## Color Mapping Logic

The system maps extracted colors to semantic roles using the following strategy:

### Color Selection Process

1. **Color Extraction**: Uses color quantization to identify dominant colors
2. **Color Analysis**: Converts colors to HSL space for better analysis
3. **Background Selection**: Finds suitable neutral colors for backgrounds
4. **Text Colors**: Selects high-contrast colors suitable for text
5. **Accent Colors**: Identifies saturated colors for semantic roles

### Semantic Role Mapping

Colors are mapped to roles based on hue ranges:

| Role | Hue Range | Color Theory |
|------|-----------|--------------|
| Error | 0°-60° | Red range for alerts |
| Warning | 30°-90° | Orange/Yellow for cautions |
| Success | 90°-150° | Green range for positive actions |
| Info | 180°-240° | Blue/Cyan for information |
| Code | 240°-300° | Blue/Purple for code elements |
| Emphasis | 300°-360° | Purple/Magenta for emphasis |

### Theme Type Detection

The system automatically determines whether to generate a light or dark theme based on:

- **Weighted average lightness** of all extracted colors
- **Frequency-based weighting** (more common colors have more influence)
- **Configurable threshold** (default: 0.7)

## Generated Theme Structure

Generated themes follow the standard thag theme format:

```toml
name = "image-sunset"
description = "Generated from image analysis"
term_bg_luma = "dark"
min_color_support = "truecolor"
backgrounds = ["#202020"]
bg_rgbs = [[32, 32, 32]]

[palette.normal]
rgb = [96, 48, 0]

[palette.error]
rgb = [112, 48, 16]

[palette.success]
rgb = [96, 48, 0]

# ... additional palette entries
```

## CLI Integration

While the CLI integration is in development due to dependency conflicts, the functionality is complete and tested. A future release will include:

```bash
# Planned CLI usage (in development)
thag_gen_theme image.jpg --light --name my-theme --output theme.toml
```

## Implementation Details

### Dependencies

The image theme generation feature requires the following optional dependencies:

```toml
[dependencies]
image = { version = "0.25", optional = true }
kmeans_colors = { version = "0.7", optional = true }
auto-palette = { version = "0.9", optional = true }
palette = { version = "0.7", optional = true }
```

Enable with the `image_themes` feature:

```toml
thag_styling = { version = "0.2.0", features = ["image_themes"] }
```

### Color Analysis Algorithm

1. **Image Loading**: Load and decode the image
2. **Color Quantization**: Reduce colors to manageable set (default: 16 colors)
3. **Frequency Analysis**: Calculate color frequencies in the image
4. **Color Space Conversion**: Convert to HSL for better analysis
5. **Categorization**: Group colors by suitability (text, accent, background)
6. **Role Assignment**: Map colors to semantic roles using hue ranges
7. **Theme Assembly**: Create final theme structure

### Performance Considerations

- Color extraction is optimized for typical image sizes
- Memory usage scales with image dimensions and color count
- Processing time is generally under 1 second for typical images
- Larger images may be automatically downsampled for performance

## Examples and Use Cases

### Use Case 1: Brand Color Themes

Extract colors from brand logos or promotional materials to create consistent terminal themes:

```rust
let brand_theme = generator.generate_from_file("logo.png")?;
```

### Use Case 2: Seasonal Themes

Create themes based on seasonal imagery:

```rust
let autumn_config = ImageThemeConfig {
    theme_name_prefix: Some("autumn".to_string()),
    color_count: 12,
    ..Default::default()
};
let autumn_theme = generator.generate_from_file("autumn-leaves.jpg")?;
```

### Use Case 3: Artwork-Inspired Themes

Generate themes from digital artwork or photography:

```rust
let art_theme = generator.generate_from_file("abstract-art.png")?;
```

## Future Enhancements

Planned improvements for future releases:

- **Advanced clustering algorithms** for better color extraction
- **Manual color adjustment** interface
- **Batch processing** for multiple images
- **Theme refinement tools** for fine-tuning results
- **Integration with terminal emulator themes** beyond thag
- **Web interface** for online theme generation

## Testing

The feature includes comprehensive tests:

```bash
# Run image theme generation tests
cd thag_rs/thag_styling
cargo test --features image_themes image_themes

# Run the example for visual verification
cargo run --example image_theme_generation --features image_themes
```

## Troubleshooting

### Common Issues

**Issue**: "Feature image_themes not enabled"
**Solution**: Ensure you've enabled the feature in your Cargo.toml

**Issue**: "Image format not supported"
**Solution**: Check that your image is in a supported format (PNG, JPEG, GIF, BMP, TIFF, WebP)

**Issue**: "Colors too similar/not enough variety"
**Solution**: Try increasing the `color_count` parameter or using a more colorful source image

### Best Practices

1. **Use high-contrast images** for better color extraction
2. **Avoid very dark or very light images** unless forcing theme type
3. **Images with 5-15 distinct colors** work best
4. **Test generated themes** in your terminal before adopting
5. **Adjust saturation_threshold** for more/fewer accent colors

## Contributing

To contribute to the image theme generation feature:

1. Ensure you have the `image_themes` feature enabled
2. Add tests for new functionality
3. Update documentation for any new configuration options
4. Test with various image types and sizes

The feature is implemented in `thag_rs/thag_styling/src/image_themes/mod.rs` with examples in the `examples/` directory.