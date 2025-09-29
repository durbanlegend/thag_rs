/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Export `thag_styling` themes to multiple terminal emulator formats
///
/// This tool exports `thag_styling` theme files to the following terminal emulator formats:
/// `Alacritty`, `iTerm2`, `Kitty`, `Konsole`, `Mintty`, `WezTerm` and `Windows Terminal`.
/// Themes are exported to organized subdirectories in ./`exported_themes`/. It also
/// optionally displays instructions for installing them into the respective emulators.
//# Purpose: Export thag themes to multiple terminal emulator formats and display further instructions.
//# Categories: color, styling, terminal, theming, tools
use inquire::set_global_render_config;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use thag_styling::{
    export_theme_to_file, file_navigator, generate_installation_instructions,
    themed_inquire_config, ExportFormat, Styleable, TermAttributes, Theme,
};

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "🎨 {} - Terminal Theme Exporter",
        "thag_gen_terminal_themes".info()
    );
    println!("{}", "=".repeat(70));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Select theme file(s)
    let theme_files = select_theme_files(&mut navigator)?;

    if theme_files.is_empty() {
        println!("❌ No theme files selected. Exiting.");
        return Ok(());
    }

    // Get export configuration
    let export_config = get_export_configuration()?;

    // Create base export directory
    let export_base_dir = PathBuf::from("./exported_themes");
    fs::create_dir_all(&export_base_dir)?;

    println!(
        "\n🚀 Exporting {} theme(s) to terminal formats...",
        theme_files.len()
    );
    println!();

    let mut total_exported = 0;
    let mut failed_exports = Vec::new();

    // Process each theme file
    for theme_file in &theme_files {
        match process_theme_file(theme_file, &export_base_dir, &export_config) {
            Ok(count) => {
                total_exported += count;
                println!(
                    "   ✅ {}: {} formats",
                    theme_file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .success(),
                    count.to_string().info()
                );
            }
            Err(e) => {
                let error_msg = e.to_string();
                failed_exports.push((theme_file.clone(), e));
                println!(
                    "   ❌ {}: {}",
                    theme_file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .error(),
                    error_msg.error()
                );
            }
        }
    }

    // Summary
    println!();
    println!("📊 {} Summary:", "Export".info());
    println!(
        "   Total themes processed: {}",
        theme_files.len().to_string().info()
    );
    println!(
        "   Total formats exported: {}",
        total_exported.to_string().success()
    );
    println!(
        "   Failed exports: {}",
        failed_exports.len().to_string().error()
    );

    if !failed_exports.is_empty() {
        println!("\n❌ {} Failures:", "Export".error());
        for (file, error) in &failed_exports {
            println!(
                "   • {}: {}",
                file.file_name().unwrap_or_default().to_string_lossy(),
                error
            );
        }
    }

    println!(
        "\n📁 Exported themes location: {}",
        export_base_dir.display().to_string().success()
    );

    // Show installation instructions if requested
    if export_config.show_instructions {
        show_installation_instructions(&export_config.formats);
    }

    println!("\n🎉 Theme export completed!");
    Ok(())
}

#[derive(Debug, Clone)]
struct ExportConfiguration {
    formats: Vec<ExportFormat>,
    show_instructions: bool,
}

/// Select theme files using file navigator
fn select_theme_files(navigator: &mut FileNavigator) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect, Select, Text};

    let selection_options = vec![
        "Select individual theme files",
        "Select all themes from a directory",
        "Select built-in theme by name",
        "Browse built-in themes interactively",
    ];

    // Make an attempt to find the most likely path
    let _ = navigator.navigate_to_path("thag_styling/themes/built_in");

    let selection_method =
        Select::new("How would you like to select themes?", selection_options).prompt()?;

    match selection_method {
        "Select individual theme files" => {
            // let extensions = &["toml", "TOML"];
            let mut selected_files = Vec::new();

            loop {
                println!("\n📁 Select a `thag_styling` theme file (.toml format):");
                if let Ok(file) = select_file(navigator, Some("toml"), false) {
                    selected_files.push(file);
                    let add_more = Confirm::new("Add another theme file?")
                        .with_default(false)
                        .prompt()?;
                    if !add_more {
                        break;
                    }
                } else {
                    if selected_files.is_empty() {
                        return Ok(vec![]);
                    }
                    break;
                }
            }
            Ok(selected_files)
        }
        "Select all themes from a directory" => {
            println!("\n📁 Select directory containing theme files:");
            match select_directory(navigator, true) {
                Ok(dir) => {
                    let theme_files = find_theme_files_in_directory(&dir)?;

                    if theme_files.is_empty() {
                        println!("❌ No .toml theme files found in directory");
                        return Ok(vec![]);
                    }

                    // Let user select which files to include
                    let file_names: Vec<String> = theme_files
                        .iter()
                        .map(|p| {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        })
                        .collect();

                    let selected_names =
                        MultiSelect::new("Select theme files to export:", file_names).prompt()?;

                    let selected_files = theme_files
                        .into_iter()
                        .filter(|f| {
                            let name = f.file_name().unwrap_or_default().to_string_lossy();
                            selected_names.contains(&name.to_string())
                        })
                        .collect();

                    Ok(selected_files)
                }
                Err(_) => Ok(vec![]),
            }
        }
        "Select built-in theme by name" => {
            let theme_name = Text::new("Enter built-in theme name:")
                .with_help_message("e.g., 'thag-vibrant-dark', 'dracula_official'")
                .prompt()?;

            // Try to load the built-in theme and create a temporary file
            match Theme::get_builtin(&theme_name) {
                Ok(theme) => {
                    let temp_file = std::env::temp_dir().join(format!("{theme_name}.toml"));
                    let toml_content = thag_styling::theme_to_toml(&theme)
                        .map_err(|e| format!("Failed to serialize theme: {e}"))?;

                    fs::write(&temp_file, toml_content)?;
                    Ok(vec![temp_file])
                }
                Err(e) => {
                    println!("❌ Failed to load built-in theme '{theme_name}': {e}");
                    Ok(vec![])
                }
            }
        }
        "Browse built-in themes interactively" => {
            // Use interactive theme browser
            let selected_themes = select_themes_interactively()?;

            // Convert themes to temporary files for processing
            let mut temp_files = Vec::new();
            for (theme_name, theme) in selected_themes {
                let temp_file = std::env::temp_dir().join(format!("{theme_name}.toml"));
                let toml_content = thag_styling::theme_to_toml(&theme)
                    .map_err(|e| format!("Failed to serialize theme: {e}"))?;

                fs::write(&temp_file, toml_content)?;
                temp_files.push(temp_file);
            }

            Ok(temp_files)
        }
        _ => Ok(vec![]),
    }
}

/// Find all theme files in a directory
fn find_theme_files_in_directory(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut theme_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "toml" || ext == "TOML" {
                    theme_files.push(path);
                }
            }
        }
    }

    theme_files.sort();
    Ok(theme_files)
}

/// Interactive theme browser similar to `thag_show_themes`
fn select_themes_interactively() -> Result<Vec<(String, Theme)>, Box<dyn Error>> {
    use inquire::MultiSelect;

    // Set up themed inquire config
    set_global_render_config(themed_inquire_config());

    // Initialize terminal attributes for theming
    let _term_attrs = TermAttributes::get_or_init();

    let mut themes = Theme::list_builtin();
    themes.sort();

    // Create theme options with descriptions
    let theme_options: Vec<String> = themes
        .iter()
        .map(|theme_name| {
            let theme = Theme::get_builtin(theme_name).unwrap_or_else(|_| {
                Theme::get_builtin("none").expect("Failed to load fallback theme")
            });
            format!("{} - {}", theme_name, theme.description)
        })
        .collect();

    println!("\n🎨 {} Built-in themes browser", "Interactive".info());
    println!("{}", "═".repeat(50));

    let selected_options = MultiSelect::new("Select themes to export:", theme_options)
        .with_page_size(15)
        .with_help_message("Space to select • ↑↓ to navigate • Enter to confirm")
        .prompt()?;

    let mut selected_themes = Vec::new();
    for selected_option in &selected_options {
        // Extract theme name (before the " - " separator)
        let theme_name = selected_option
            .split(" - ")
            .next()
            .unwrap_or(selected_option);

        match Theme::get_builtin(theme_name) {
            Ok(theme) => {
                println!("   📋 Added: {}", theme.name.info());
                selected_themes.push((theme_name.to_string(), theme));
            }
            Err(e) => {
                println!("   ❌ Failed to load theme '{theme_name}': {e}");
            }
        }
    }

    if selected_themes.is_empty() {
        println!("❌ No themes selected");
        return Ok(vec![]);
    }

    println!("\n✅ Selected {} themes for export", selected_themes.len());
    Ok(selected_themes)
    // Ok(selected_options)
}

/// Get export configuration from user
fn get_export_configuration() -> Result<ExportConfiguration, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect};

    // Get available formats
    let all_formats = get_all_export_formats();
    let format_names: Vec<String> = all_formats
        .iter()
        .map(|f| format!("{} ({})", f.format_name(), f.file_extension()))
        .collect();

    let format_names_len = format_names.len();
    let selected_format_names = MultiSelect::new("Select export formats:", format_names.clone())
        .with_default(&(0..format_names_len).collect::<Vec<_>>()) // Select all by default
        .prompt()?;

    let selected_formats: Vec<ExportFormat> = all_formats
        .into_iter()
        .enumerate()
        .filter(|(i, _)| selected_format_names.contains(&format_names[*i]))
        .map(|(_, format)| format)
        .collect();

    if selected_formats.is_empty() {
        return Err("No formats selected".into());
    }

    let show_instructions = Confirm::new("Show installation instructions after export?")
        .with_default(true)
        .prompt()?;

    Ok(ExportConfiguration {
        formats: selected_formats,
        show_instructions,
    })
}

/// Process a single theme file
fn process_theme_file(
    theme_file: &Path,
    export_base_dir: &Path,
    config: &ExportConfiguration,
) -> Result<usize, Box<dyn Error>> {
    // Load theme
    let theme =
        Theme::load_from_file(theme_file).map_err(|e| format!("Failed to load theme: {e}"))?;

    let theme_base_name = theme_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("theme");

    let mut exported_count = 0;

    // Export to each selected format
    for format in &config.formats {
        // Always organize by format in subdirectories
        let format_dir = match format {
            ExportFormat::Alacritty => export_base_dir.join("alacritty"),
            ExportFormat::ITerm2 => export_base_dir.join("iterm2"),
            ExportFormat::Kitty => export_base_dir.join("kitty"),
            ExportFormat::Konsole => export_base_dir.join("konsole"),
            ExportFormat::Mintty => export_base_dir.join("mintty"),
            ExportFormat::WezTerm => export_base_dir.join("wezterm"),
            ExportFormat::WindowsTerminal => export_base_dir.join("windows"),
        };

        fs::create_dir_all(&format_dir)?;

        // Use simple filenames since we have subdirectories
        let filename = format!("{}.{}", theme_base_name, format.file_extension());
        let output_path = format_dir.join(filename);

        export_theme_to_file(&theme, *format, &output_path)
            .map_err(|e| format!("Failed to export {} format: {}", format.format_name(), e))?;

        exported_count += 1;
    }

    Ok(exported_count)
}

/// Show installation instructions for selected formats with actual theme names
fn show_installation_instructions(formats: &[ExportFormat]) {
    println!("\n📖 {} Instructions:", "Installation".info());
    println!("{}", "=".repeat(70));

    for format in formats {
        println!("\n🔧 {}", format.format_name().info());
        println!("{}", "─".repeat(30));

        // Use a generic placeholder since we don't know the specific theme name here
        let instructions = generate_installation_instructions(*format, "<theme-name>");
        println!("{instructions}");

        println!(
            "\n💡 {} Replace {} with your actual theme filename",
            "Note:".warning(),
            "<theme-name>".info()
        );
    }
}

// Helper function to get all supported export formats
fn get_all_export_formats() -> Vec<ExportFormat> {
    vec![
        ExportFormat::Alacritty,
        ExportFormat::ITerm2,
        ExportFormat::Kitty,
        ExportFormat::Konsole,
        ExportFormat::Mintty,
        ExportFormat::WezTerm,
        ExportFormat::WindowsTerminal,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorSupport, Palette, TermBgLuma};

    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Export Theme".to_string(),
            filename: PathBuf::from("test_export_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![(42, 42, 42)],
            description: "Test theme for export functionality".to_string(),
        }
    }

    #[test]
    fn test_theme_file_discovery() {
        let temp_dir = std::env::temp_dir().join("thag_test_themes");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test theme files
        fs::write(temp_dir.join("theme1.toml"), "test content 1").unwrap();
        fs::write(temp_dir.join("theme2.toml"), "test content 2").unwrap();
        fs::write(temp_dir.join("not_theme.txt"), "not a theme").unwrap();

        let found_files = find_theme_files_in_directory(&temp_dir).unwrap();
        assert_eq!(found_files.len(), 2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_export_configuration_defaults() {
        let config = ExportConfiguration {
            formats: vec![ExportFormat::Alacritty, ExportFormat::WezTerm],
            show_instructions: true,
        };

        assert_eq!(config.formats.len(), 2);
        assert!(config.show_instructions);
    }

    #[test]
    fn test_all_formats_available() {
        let formats = get_all_export_formats();
        assert!(!formats.is_empty());

        // Check that we have the expected formats
        let format_names: Vec<String> = formats
            .iter()
            .map(|f| f.format_name().to_string())
            .collect();

        assert!(format_names.contains(&"Alacritty".to_string()));
        assert!(format_names.contains(&"WezTerm".to_string()));
        assert!(format_names.contains(&"Kitty".to_string()));
        assert!(format_names.contains(&"Mintty".to_string()));
    }
}
