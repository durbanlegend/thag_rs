# Multi-Format Theme Export System

## Overview

The thag_rs project now includes a comprehensive multi-format theme export system that allows users to convert thag themes to popular terminal emulator formats. This enables seamless theme sharing across different terminal applications.

## Phase 1 Implementation Status: ✅ Complete

We have successfully implemented **Phase 1** of the multi-format export system as outlined in the [multi-format terminal theme export discussion](claude/multi_format_terminal_theme_export_discussion.md).

### Supported Terminal Formats

| Terminal Emulator | Format | File Extension | Status |
|-------------------|--------|----------------|---------|
| **Alacritty** | TOML | `.toml` | ✅ Implemented |
| **WezTerm** | Lua | `.lua` | ✅ Implemented |
| **iTerm2** | JSON | `.json` | ✅ Implemented |
| **Kitty** | Config | `.conf` | ✅ Implemented |
| **Windows Terminal** | JSON | `.json` | ✅ Implemented |

## Architecture

### Core Components

1. **ThemeExporter Trait** (`thag_styling/src/exporters/mod.rs`)
   - Defines the interface for all theme exporters
   - Methods: `export_theme()`, `file_extension()`, `format_name()`

2. **ExportFormat Enum**
   - Represents all supported export formats
   - Provides unified access to format-specific functionality

3. **Individual Exporters**
   - `AlacrittyExporter`: Converts themes to Alacritty's TOML format
   - `WezTermExporter`: Converts themes to WezTerm's Lua format
   - `ITerm2Exporter`: Converts themes to iTerm2's JSON format
   - `KittyExporter`: Converts themes to Kitty's configuration format
   - `WindowsTerminalExporter`: Converts themes to Windows Terminal's JSON format

### Color Mapping Strategy

Each exporter intelligently maps thag's semantic color roles to terminal-specific color slots:

- **Error** → Red (ANSI 1)
- **Success** → Green (ANSI 2)  
- **Warning** → Yellow (ANSI 3)
- **Info** → Blue (ANSI 4)
- **Code** → Magenta (ANSI 5)
- **Normal** → White (ANSI 7)
- **Subtle** → Bright Black (ANSI 8)
- **Emphasis** → Bright colors and cursor

## Usage

### Basic Export Functions

```rust
use thag_styling::{ExportFormat, export_all_formats, export_theme_to_file};

// Export to all formats at once
let exported_files = export_all_formats(&theme, "output_dir", "theme_name")?;

// Export to specific format
export_theme_to_file(&theme, ExportFormat::Alacritty, "theme.toml")?;

// Export individual format with content
let content = ExportFormat::WezTerm.export_theme(&theme)?;
```

### Installation Instructions

The system automatically generates installation instructions for each format:

```rust
use thag_styling::generate_installation_instructions;

let instructions = generate_installation_instructions(
    ExportFormat::Alacritty, 
    "my_theme.toml"
);
println!("{}", instructions);
```

## Demo Programs

### 1. Multi-Format Theme Export Demo
**File**: `demo/multi_format_theme_export.rs`

Demonstrates exporting a programmatically created theme to all supported formats.

```bash
THAG_DEV_PATH=$PWD cargo run -- demo/multi_format_theme_export.rs
```

**Features**:
- Creates a vibrant sample theme
- Exports to all 5 terminal formats
- Shows installation instructions for each format
- Provides content previews

### 2. Image-to-Multi-Format Theme Demo  
**File**: `demo/image_to_multi_format_theme.rs`

Shows the complete workflow: image analysis → theme generation → multi-format export.

```bash
THAG_DEV_PATH=$PWD cargo run -- demo/image_to_multi_format_theme.rs
```

**Features**:
- Analyzes an image to extract color palette
- Generates semantic color roles from extracted colors
- Exports to all terminal formats
- Shows color palette analysis
- Provides usage examples

## Format-Specific Implementation Details

### Alacritty (TOML)
- Supports primary, normal, bright, and dim color variants
- Includes cursor, selection, and search highlighting colors
- Compatible with both legacy YAML and modern TOML configs

### WezTerm (Lua)
- Full Lua module with metadata
- Comprehensive tab bar color configuration
- Split/pane border colors
- Scrollbar theming support

### iTerm2 (JSON)
- Complete color preset format with normalized float values (0.0-1.0)
- Supports all iTerm2-specific colors (badge, guide, session name)
- sRGB color space specification

### Kitty (Config)
- Simple key-value configuration format
- Mark colors for text search highlighting
- Visual bell and border color support
- Tab bar theming

### Windows Terminal (JSON)
- Proper schemes array structure for merging into settings.json
- All 16 ANSI colors with Windows Terminal naming convention
- Cursor and selection color support

## Color Conversion Features

### Intelligent Color Processing
- **True Color**: Direct RGB value export
- **256-Color**: Automatic conversion to approximate RGB
- **Basic Colors**: Fallback to 16-color ANSI mapping

### Color Enhancement
- **Brightness Adjustment**: Automatic generation of bright variants
- **Contrast Optimization**: Ensures readability across different backgrounds
- **Semantic Mapping**: Intelligent assignment of colors to semantic roles

## Testing

Each exporter includes comprehensive unit tests:

```bash
cd thag_styling
cargo test exporters
```

Tests cover:
- Color conversion accuracy
- Format-specific output validation
- Installation instruction generation
- Error handling and edge cases

## Future Enhancements (Phase 2+)

### Planned Features
- **Runtime OSC Updates**: Apply themes instantly without restart
- **Base16 Compatibility**: Generate Base16-compatible palettes
- **Terminal Detection**: Auto-detect installed terminals
- **Batch Processing**: Export multiple themes at once
- **Theme Validation**: Verify generated themes before export

### Additional Terminal Support
- Hyper terminal
- Terminator
- GNOME Terminal
- Konsole

## Integration Points

### With Image Theme Generation
```rust
// Generate theme from image and export to all formats
let theme = generate_theme_from_image("image.png")?;
let files = export_all_formats(&theme, "themes/", &theme.name)?;
```

### With Existing Theme System
```rust
// Export any existing thag theme
let theme = Theme::get_builtin("dracula")?;
export_theme_to_file(&theme, ExportFormat::Alacritty, "dracula.toml")?;
```

## File Structure

```
thag_styling/src/exporters/
├── mod.rs                    # Main module with trait and utilities
├── alacritty.rs             # Alacritty TOML exporter
├── wezterm.rs               # WezTerm Lua exporter  
├── iterm2.rs                # iTerm2 JSON exporter
├── kitty.rs                 # Kitty config exporter
└── windows_terminal.rs      # Windows Terminal JSON exporter
```

## Dependencies

- `serde_json`: JSON serialization for iTerm2 and Windows Terminal formats
- Existing thag_styling infrastructure for color handling and theme management

## Error Handling

Robust error handling throughout the export process:
- Invalid color values → Fallback to sensible defaults
- IO errors → Graceful failure with informative messages  
- Serialization errors → Detailed error context
- Missing theme data → Automatic color generation

## Performance

- **Fast Exports**: Each format exports in milliseconds
- **Efficient Color Conversion**: Cached color calculations
- **Minimal Memory Usage**: Streaming output generation
- **Parallel Export**: Multiple formats can be exported concurrently

## Conclusion

The multi-format theme export system successfully addresses the original problem of theme portability across terminal emulators. Users can now:

1. Generate themes from images using thag's image analysis
2. Export those themes to any popular terminal emulator
3. Get detailed installation instructions for each format
4. Share themes easily across different platforms and terminals

This implementation provides a solid foundation for the remaining phases of the multi-format export roadmap.