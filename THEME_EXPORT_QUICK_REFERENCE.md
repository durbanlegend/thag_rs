# Theme Export Quick Reference

## Export All Formats

```rust
use thag_styling::export_all_formats;

// Export theme to all supported formats
let files = export_all_formats(&theme, "output_dir", "my_theme")?;
```

## Export Single Format

```rust
use thag_styling::{ExportFormat, export_theme_to_file};

// Export to specific terminal
export_theme_to_file(&theme, ExportFormat::Alacritty, "my_theme.toml")?;
export_theme_to_file(&theme, ExportFormat::WezTerm, "my_theme.lua")?;
export_theme_to_file(&theme, ExportFormat::ITerm2, "my_theme.json")?;
export_theme_to_file(&theme, ExportFormat::Kitty, "my_theme.conf")?;
export_theme_to_file(&theme, ExportFormat::WindowsTerminal, "my_theme.json")?;
```

## Get Theme Content

```rust
use thag_styling::{ExportFormat, ThemeExporter};

// Get theme content as string
let alacritty_content = ExportFormat::Alacritty.export_theme(&theme)?;
let wezterm_content = ExportFormat::WezTerm.export_theme(&theme)?;
```

## Installation Instructions

```rust
use thag_styling::generate_installation_instructions;

let instructions = generate_installation_instructions(
    ExportFormat::Alacritty, 
    "my_theme.toml"
);
```

## File Extensions

| Terminal | Extension | Format |
|----------|-----------|--------|
| Alacritty | `.toml` | TOML |
| WezTerm | `.lua` | Lua |
| iTerm2 | `.json` | JSON |
| Kitty | `.conf` | Config |
| Windows Terminal | `.json` | JSON |

## Terminal Installation Paths

### Alacritty
- **Linux/macOS**: `~/.config/alacritty/themes/`
- **Windows**: `%APPDATA%\alacritty\themes\`
- **Config**: `import = ["themes/theme_name.toml"]`

### WezTerm
- **Linux/macOS**: `~/.config/wezterm/colors/`
- **Windows**: `%USERPROFILE%\.config\wezterm\colors\`
- **Config**: `config.color_scheme = "theme_name"`

### iTerm2
- **Installation**: Preferences → Profiles → Colors → Color Presets → Import
- **Format**: JSON color preset

### Kitty
- **Linux/macOS**: `~/.config/kitty/themes/`
- **Windows**: `%APPDATA%\kitty\themes\`
- **Config**: `include themes/theme_name.conf`

### Windows Terminal
- **Installation**: Settings → Open JSON → Add to "schemes" array
- **Config**: `"colorScheme": "theme_name"`

## Complete Workflow Example

```rust
use thag_styling::{
    generate_theme_from_image,
    export_all_formats,
    generate_installation_instructions,
    ExportFormat
};

// 1. Generate theme from image
let theme = generate_theme_from_image("photo.jpg")?;

// 2. Export to all formats
let files = export_all_formats(&theme, "themes/", &theme.name)?;

// 3. Print installation instructions
for format in ExportFormat::all() {
    let filename = format!("{}.{}", theme.name, format.file_extension());
    let instructions = generate_installation_instructions(*format, &filename);
    println!("=== {} ===", format.format_name());
    println!("{}", instructions);
}
```

## Demo Scripts

Run these to see the export system in action:

```bash
# Basic multi-format export demo
THAG_DEV_PATH=$PWD cargo run -- demo/multi_format_theme_export.rs

# Image-to-theme export demo  
THAG_DEV_PATH=$PWD cargo run -- demo/image_to_multi_format_theme.rs
```

## Color Mapping

thag semantic roles map to terminal colors as follows:

- **error** → Red (ANSI 1)
- **success** → Green (ANSI 2)
- **warning** → Yellow (ANSI 3)
- **info** → Blue (ANSI 4)
- **code** → Magenta (ANSI 5)
- **normal** → White (ANSI 7)
- **subtle** → Bright Black (ANSI 8)
- **emphasis** → Used for cursor and bright variants

## Error Handling

All export functions return `Result` types. Handle errors appropriately:

```rust
match export_all_formats(&theme, "output", "theme_name") {
    Ok(files) => println!("Exported {} files", files.len()),
    Err(e) => eprintln!("Export failed: {}", e),
}
```

## Testing

```bash
# Test all exporters
cargo test exporters

# Test specific format
cargo test alacritty
cargo test wezterm
```
