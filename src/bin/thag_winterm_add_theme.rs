/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto" }
dirs = "5.0"
serde_json = "1.0"
*/

/// Windows Terminal theme installer with file navigator
///
/// This Windows-only tool installs thag themes into Windows Terminal by
/// adding theme schemes to the settings.json configuration file. Supports
/// selecting individual themes or entire directories of themes.
//# Purpose: Install thag themes for Windows Terminal
//# Categories: color, styling, terminal, theming, tools, windows
use colored::Colorize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use thag_proc_macros::file_navigator;
use thag_rs::{cprtln, Role, ThagResult};
use thag_styling::{ExportFormat, Theme};

file_navigator! {}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), Box<dyn Error>> {
    println!("❌ This tool is only available on Windows systems.");
    println!("   Windows Terminal is not available on other platforms.");
    Ok(())
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "🖥️  {} - Windows Terminal Theme Installer",
        "thag_winterm_add_theme".bright_blue()
    );
    println!("{}", "=".repeat(70).dimmed());
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Get Windows Terminal settings path
    let settings_path = get_windows_terminal_settings_path()?;

    println!("📁 Windows Terminal configuration:");
    println!(
        "   Settings file: {}",
        settings_path.display().to_string().bright_cyan()
    );

    // Check if settings file exists
    if !settings_path.exists() {
        println!("❌ Windows Terminal settings.json not found.");
        println!("   Please ensure Windows Terminal is installed and has been run at least once.");
        return Ok(());
    }

    println!("   Status: {}", "Found".bright_green());
    println!();

    // Select themes to install
    let themes = select_themes_for_installation(&mut navigator)?;

    if themes.is_empty() {
        println!("❌ No themes selected for installation.");
        return Ok(());
    }

    println!("🎨 Installing {} theme(s)...", themes.len());
    println!();

    // Load current settings
    let mut settings = load_windows_terminal_settings(&settings_path)?;

    // Backup settings file
    backup_settings_file(&settings_path)?;

    let mut added_schemes = Vec::new();
    let mut installation_errors = Vec::new();

    // Add each theme as a color scheme
    for theme in &themes {
        match add_theme_to_settings(theme, &mut settings) {
            Ok(scheme_name) => {
                added_schemes.push((theme.name.clone(), scheme_name));
                println!("   ✅ {}", theme.name.bright_green());
            }
            Err(e) => {
                installation_errors.push((theme.name.clone(), e));
                println!("   ❌ {}: {}", theme.name.bright_red(), e.to_string().red());
            }
        }
    }

    // Save updated settings
    if !added_schemes.is_empty() {
        save_windows_terminal_settings(&settings_path, &settings)?;
        println!("\n✅ Windows Terminal settings updated successfully");
    }

    // Show summary and instructions
    show_installation_summary(&added_schemes, &installation_errors);
    show_usage_instructions(&added_schemes);

    println!("\n🎉 Theme installation completed!");
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_windows_terminal_settings_path() -> Result<PathBuf, Box<dyn Error>> {
    let local_app_data = dirs::data_local_dir().ok_or("Could not find local app data directory")?;

    let settings_path = local_app_data
        .join("Packages")
        .join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
        .join("LocalState")
        .join("settings.json");

    Ok(settings_path)
}

#[cfg(target_os = "windows")]
fn select_themes_for_installation(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect, Select, Text};

    let selection_options = vec![
        "Select theme files (.toml)",
        "Select all themes from directory",
        "Select exported Windows Terminal themes (.json)",
        "Install built-in theme by name",
        "Select from multiple built-in themes",
    ];

    let selection_method =
        Select::new("How would you like to select themes?", selection_options).prompt()?;

    match selection_method {
        "Select theme files (.toml)" => select_individual_toml_themes(navigator),
        "Select all themes from directory" => select_themes_from_directory(navigator),
        "Select exported Windows Terminal themes (.json)" => select_exported_json_themes(navigator),
        "Install built-in theme by name" => select_builtin_theme_by_name(),
        "Select from multiple built-in themes" => select_multiple_builtin_themes(),
        _ => Ok(vec![]),
    }
}

#[cfg(target_os = "windows")]
fn select_individual_toml_themes(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::Confirm;

    let extensions = vec!["toml", "TOML"];
    let mut selected_themes = Vec::new();

    loop {
        println!("\n📁 Select a theme file (.toml format):");
        match select_file(navigator, Some("toml"), false) {
            Ok(theme_file) => {
                match Theme::load_from_file(&theme_file) {
                    Ok(theme) => {
                        println!(
                            "   📋 Loaded: {} - {}",
                            theme.name.bright_cyan(),
                            theme.description.dimmed()
                        );
                        selected_themes.push(theme);
                    }
                    Err(e) => {
                        println!("   ❌ Failed to load theme: {}", e.to_string().red());
                        continue;
                    }
                }

                let add_more = Confirm::new("Add another theme file?")
                    .with_default(false)
                    .prompt()?;

                if !add_more {
                    break;
                }
            }
            Err(_) => {
                if selected_themes.is_empty() {
                    return Ok(vec![]);
                }
                break;
            }
        }
    }

    Ok(selected_themes)
}

#[cfg(target_os = "windows")]
fn select_themes_from_directory(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::MultiSelect;

    println!("\n📁 Select directory containing theme files (.toml):");
    match select_directory(navigator, true) {
        Ok(theme_dir) => {
            let theme_files = find_theme_files_in_directory(&theme_dir)?;

            if theme_files.is_empty() {
                println!("❌ No .toml theme files found in directory");
                return Ok(vec![]);
            }

            let mut themes = Vec::new();
            for theme_file in &theme_files {
                match Theme::load_from_file(theme_file) {
                    Ok(theme) => themes.push(theme),
                    Err(e) => {
                        println!(
                            "⚠️  Skipping {}: {}",
                            theme_file.file_name().unwrap_or_default().to_string_lossy(),
                            e.to_string().yellow()
                        );
                    }
                }
            }

            // Let user select which themes to install
            if themes.len() > 1 {
                let theme_names: Vec<String> = themes
                    .iter()
                    .map(|t| format!("{} - {}", t.name, t.description))
                    .collect();

                let selected_names = MultiSelect::new("Select themes to install:", theme_names)
                    .with_default(&(0..theme_names.len()).collect::<Vec<_>>())
                    .prompt()?;

                let selected_themes = themes
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| selected_names.contains(&theme_names[*i]))
                    .map(|(_, theme)| theme)
                    .collect();

                Ok(selected_themes)
            } else {
                Ok(themes)
            }
        }
        Err(_) => Ok(vec![]),
    }
}

#[cfg(target_os = "windows")]
fn select_exported_json_themes(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect};

    println!("\n📁 Select directory containing exported Windows Terminal themes (.json):");
    match select_directory(navigator, true) {
        Ok(json_dir) => {
            let json_files = find_json_files_in_directory(&json_dir)?;

            if json_files.is_empty() {
                println!("❌ No .json theme files found in directory");
                return Ok(vec![]);
            }

            // Let user select which JSON files to use
            let file_names: Vec<String> = json_files
                .iter()
                .map(|p| {
                    p.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .collect();

            let selected_names = MultiSelect::new("Select theme files to install:", file_names)
                .with_default(&(0..file_names.len()).collect::<Vec<_>>())
                .prompt()?;

            let selected_files: Vec<_> = json_files
                .into_iter()
                .enumerate()
                .filter(|(i, _)| selected_names.contains(&file_names[*i]))
                .map(|(_, file)| file)
                .collect();

            // Convert JSON schemes to Theme objects (simplified)
            let mut themes = Vec::new();
            for json_file in selected_files {
                match load_theme_from_json(&json_file) {
                    Ok(theme) => {
                        println!("   📋 Loaded JSON theme: {}", theme.name.bright_cyan());
                        themes.push(theme);
                    }
                    Err(e) => {
                        println!(
                            "   ⚠️  Failed to load {}: {}",
                            json_file.file_name().unwrap_or_default().to_string_lossy(),
                            e.to_string().yellow()
                        );
                    }
                }
            }

            Ok(themes)
        }
        Err(_) => Ok(vec![]),
    }
}

#[cfg(target_os = "windows")]
fn select_builtin_theme_by_name() -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::Text;

    let theme_name = Text::new("Enter built-in theme name:")
        .with_help_message("e.g., 'thag-vibrant-dark', 'dracula_official'")
        .prompt()?;

    match Theme::get_builtin(&theme_name) {
        Ok(theme) => {
            println!(
                "📋 Found: {} - {}",
                theme.name.bright_cyan(),
                theme.description.dimmed()
            );
            Ok(vec![theme])
        }
        Err(e) => {
            println!("❌ Failed to load built-in theme '{}': {}", theme_name, e);
            Ok(vec![])
        }
    }
}

#[cfg(target_os = "windows")]
fn select_multiple_builtin_themes() -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::MultiSelect;

    println!("\n📚 {} Built-in themes:", "Available".bright_blue());

    let common_themes = vec![
        "thag-vibrant-dark",
        "thag-vibrant-light",
        "thag-morning-coffee-dark",
        "thag-morning-coffee-light",
        "dracula_official",
        "gruvbox_dark",
        "gruvbox_light",
        "solarized_dark",
        "solarized_light",
    ];

    let mut available_themes = Vec::new();
    let mut theme_display_names = Vec::new();

    for theme_name in &common_themes {
        match Theme::get_builtin(theme_name) {
            Ok(theme) => {
                theme_display_names.push(format!("{} - {}", theme.name, theme.description));
                available_themes.push(theme);
            }
            Err(_) => {
                // Skip unavailable themes
            }
        }
    }

    if available_themes.is_empty() {
        println!("❌ No built-in themes available");
        return Ok(vec![]);
    }

    let selected_names =
        MultiSelect::new("Select themes to install:", theme_display_names.clone()).prompt()?;

    let selected_themes = available_themes
        .into_iter()
        .enumerate()
        .filter(|(i, _)| selected_names.contains(&theme_display_names[*i]))
        .map(|(_, theme)| theme)
        .collect();

    Ok(selected_themes)
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn find_json_files_in_directory(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut json_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" || ext == "JSON" {
                    json_files.push(path);
                }
            }
        }
    }

    json_files.sort();
    Ok(json_files)
}

#[cfg(target_os = "windows")]
fn load_theme_from_json(json_file: &Path) -> Result<Theme, Box<dyn Error>> {
    // This is a simplified approach - we create a minimal Theme object
    // from the JSON scheme name for installation purposes
    let content = fs::read_to_string(json_file)?;
    let json_data: serde_json::Value = serde_json::from_str(&content)?;

    let scheme_name = json_data
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown Theme")
        .to_string();

    // Create a minimal theme object - the actual color data will be read from JSON during installation
    use std::path::PathBuf;
    use thag_styling::{ColorSupport, Palette, TermBgLuma};

    Ok(Theme {
        name: scheme_name.clone(),
        filename: PathBuf::from(json_file),
        is_builtin: false,
        term_bg_luma: TermBgLuma::Dark, // We'll determine this from the JSON
        min_color_support: ColorSupport::TrueColor,
        palette: Palette::default(),
        backgrounds: vec!["#000000".to_string()],
        bg_rgbs: vec![(0, 0, 0)],
        description: format!(
            "Imported from {}",
            json_file.file_name().unwrap_or_default().to_string_lossy()
        ),
    })
}

#[cfg(target_os = "windows")]
fn load_windows_terminal_settings(
    settings_path: &Path,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let content = fs::read_to_string(settings_path)?;
    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;
    Ok(settings)
}

#[cfg(target_os = "windows")]
fn backup_settings_file(settings_path: &Path) -> Result<(), Box<dyn Error>> {
    let backup_path = settings_path.with_extension("bak");
    fs::copy(settings_path, &backup_path)?;
    println!(
        "💾 Created backup: {}",
        backup_path.display().to_string().dimmed()
    );
    Ok(())
}

#[cfg(target_os = "windows")]
fn add_theme_to_settings(
    theme: &Theme,
    settings: &mut serde_json::Value,
) -> Result<String, Box<dyn Error>> {
    // Generate Windows Terminal color scheme from theme
    let color_scheme = if theme.filename.extension().and_then(|s| s.to_str()) == Some("json") {
        // Load existing JSON scheme
        let json_content = fs::read_to_string(&theme.filename)?;
        serde_json::from_str(&json_content)?
    } else {
        // Generate scheme from thag theme
        generate_windows_terminal_scheme(theme)?
    };

    // Get scheme name
    let scheme_name = color_scheme
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("Color scheme missing name")?
        .to_string();

    // Ensure schemes array exists
    let schemes = settings
        .as_object_mut()
        .ok_or("Settings is not an object")?
        .entry("schemes")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    let schemes_array = schemes.as_array_mut().ok_or("Schemes is not an array")?;

    // Check if scheme already exists
    let scheme_exists = schemes_array
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(&scheme_name));

    if !scheme_exists {
        schemes_array.push(color_scheme);
    }

    Ok(scheme_name)
}

#[cfg(target_os = "windows")]
fn generate_windows_terminal_scheme(theme: &Theme) -> Result<serde_json::Value, Box<dyn Error>> {
    use serde_json::json;

    let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));
    let bg_hex = format!("#{:02X}{:02X}{:02X}", bg_color.0, bg_color.1, bg_color.2);

    // Extract colors with fallbacks
    let normal_color = extract_rgb(&theme.palette.normal).unwrap_or((192, 192, 192));
    let error_color = extract_rgb(&theme.palette.error).unwrap_or((255, 0, 0));
    let success_color = extract_rgb(&theme.palette.success).unwrap_or((0, 255, 0));
    let warning_color = extract_rgb(&theme.palette.warning).unwrap_or((255, 255, 0));
    let info_color = extract_rgb(&theme.palette.info).unwrap_or((0, 0, 255));
    let heading1_color = extract_rgb(&theme.palette.heading1).unwrap_or((255, 0, 255));
    let heading3_color = extract_rgb(&theme.palette.heading3).unwrap_or((0, 255, 255));
    let subtle_color = extract_rgb(&theme.palette.subtle).unwrap_or((128, 128, 128));

    let scheme = json!({
        "name": theme.name,
        "black": format!("#{:02X}{:02X}{:02X}", bg_color.0, bg_color.1, bg_color.2),
        "red": format!("#{:02X}{:02X}{:02X}", error_color.0, error_color.1, error_color.2),
        "green": format!("#{:02X}{:02X}{:02X}", success_color.0, success_color.1, success_color.2),
        "yellow": format!("#{:02X}{:02X}{:02X}", warning_color.0, warning_color.1, warning_color.2),
        "blue": format!("#{:02X}{:02X}{:02X}", info_color.0, info_color.1, info_color.2),
        "purple": format!("#{:02X}{:02X}{:02X}", heading1_color.0, heading1_color.1, heading1_color.2),
        "cyan": format!("#{:02X}{:02X}{:02X}", heading3_color.0, heading3_color.1, heading3_color.2),
        "white": format!("#{:02X}{:02X}{:02X}", normal_color.0, normal_color.1, normal_color.2),
        "brightBlack": format!("#{:02X}{:02X}{:02X}", subtle_color.0, subtle_color.1, subtle_color.2),
        "brightRed": format!("#{:02X}{:02X}{:02X}", brighten_color(error_color).0, brighten_color(error_color).1, brighten_color(error_color).2),
        "brightGreen": format!("#{:02X}{:02X}{:02X}", brighten_color(success_color).0, brighten_color(success_color).1, brighten_color(success_color).2),
        "brightYellow": format!("#{:02X}{:02X}{:02X}", brighten_color(warning_color).0, brighten_color(warning_color).1, brighten_color(warning_color).2),
        "brightBlue": format!("#{:02X}{:02X}{:02X}", brighten_color(info_color).0, brighten_color(info_color).1, brighten_color(info_color).2),
        "brightPurple": format!("#{:02X}{:02X}{:02X}", brighten_color(heading1_color).0, brighten_color(heading1_color).1, brighten_color(heading1_color).2),
        "brightCyan": format!("#{:02X}{:02X}{:02X}", brighten_color(heading3_color).0, brighten_color(heading3_color).1, brighten_color(heading3_color).2),
        "brightWhite": format!("#{:02X}{:02X}{:02X}", brighten_color(normal_color).0, brighten_color(normal_color).1, brighten_color(normal_color).2),
        "background": bg_hex,
        "foreground": format!("#{:02X}{:02X}{:02X}", normal_color.0, normal_color.1, normal_color.2)
    });

    Ok(scheme)
}

#[cfg(target_os = "windows")]
fn save_windows_terminal_settings(
    settings_path: &Path,
    settings: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let pretty_json = serde_json::to_string_pretty(settings)?;
    fs::write(settings_path, pretty_json)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn show_installation_summary(
    added_schemes: &[(String, String)],
    errors: &[(String, Box<dyn Error>)],
) {
    println!("\n📊 {} Summary:", "Installation".bright_blue());
    println!(
        "   Successfully added: {}",
        added_schemes.len().to_string().bright_green()
    );
    println!(
        "   Failed installations: {}",
        errors.len().to_string().bright_red()
    );

    if !added_schemes.is_empty() {
        println!("\n✅ {} Color Schemes:", "Added".bright_green());
        for (theme_name, scheme_name) in added_schemes {
            println!(
                "   • {} ({})",
                theme_name.bright_cyan(),
                scheme_name.dimmed()
            );
        }
    }

    if !errors.is_empty() {
        println!("\n❌ {} Failures:", "Installation".bright_red());
        for (theme_name, error) in errors {
            println!("   • {}: {}", theme_name, error.to_string().red());
        }
    }
}

#[cfg(target_os = "windows")]
fn show_usage_instructions(added_schemes: &[(String, String)]) {
    if added_schemes.is_empty() {
        return;
    }

    println!("\n💡 {} to Use:", "How".bright_blue());
    println!("1. Open Windows Terminal");
    println!("2. Open Settings (Ctrl+,)");
    println!("3. Go to Profiles → Defaults (or specific profile)");
    println!("4. Under Appearance, select Color scheme:");

    for (_, scheme_name) in added_schemes {
        println!("   • {}", scheme_name.bright_cyan());
    }

    println!("\n📖 {} Steps:", "Additional".bright_blue());
    println!("• Restart Windows Terminal to ensure changes take effect");
    println!("• You can also set color schemes per profile for different use cases");
    println!(
        "• Use {} to generate more theme formats",
        "thag_gen_terminal_themes".bright_cyan()
    );
}

#[cfg(target_os = "windows")]
fn extract_rgb(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

#[cfg(target_os = "windows")]
fn brighten_color((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    let factor = 1.3;
    (
        ((r as f32 * factor).min(255.0)) as u8,
        ((g as f32 * factor).min(255.0)) as u8,
        ((b as f32 * factor).min(255.0)) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::*;
        use std::path::PathBuf;
        use thag_styling::{ColorInfo, ColorSupport, Palette, Style, TermBgLuma};

        fn create_test_theme() -> Theme {
            let mut palette = Palette::default();
            palette.normal = Style::fg(ColorInfo::rgb(220, 220, 220));
            palette.error = Style::fg(ColorInfo::rgb(255, 100, 100));

            Theme {
                name: "Test Windows Terminal Theme".to_string(),
                filename: PathBuf::from("test_winterm_theme.toml"),
                is_builtin: false,
                term_bg_luma: TermBgLuma::Dark,
                min_color_support: ColorSupport::TrueColor,
                palette,
                backgrounds: vec!["#2a2a2a".to_string()],
                bg_rgbs: vec![(42, 42, 42)],
                description: "Test theme for Windows Terminal".to_string(),
            }
        }

        #[test]
        fn test_extract_rgb() {
            let style = Style::fg(ColorInfo::rgb(255, 128, 64));
            let rgb = extract_rgb(&style);
            assert_eq!(rgb, Some((255, 128, 64)));
        }

        #[test]
        fn test_brighten_color() {
            let original = (100, 150, 200);
            let brightened = brighten_color(original);

            assert!(brightened.0 >= original.0);
            assert!(brightened.1 >= original.1);
            assert!(brightened.2 >= original.2);
        }

        #[test]
        fn test_generate_windows_terminal_scheme() {
            let theme = create_test_theme();
            let result = generate_windows_terminal_scheme(&theme);

            assert!(result.is_ok());
            let scheme = result.unwrap();
            assert!(scheme.get("name").is_some());
            assert!(scheme.get("background").is_some());
            assert!(scheme.get("foreground").is_some());
            assert!(scheme.get("black").is_some());
            assert!(scheme.get("red").is_some());
        }

        #[test]
        fn test_theme_file_discovery() {
            let temp_dir = std::env::temp_dir().join("thag_test_winterm_themes");
            fs::create_dir_all(&temp_dir).unwrap();

            // Create test files
            fs::write(temp_dir.join("theme1.toml"), "test theme 1").unwrap();
            fs::write(temp_dir.join("theme2.json"), r#"{"name": "test"}"#).unwrap();
            fs::write(temp_dir.join("not_theme.txt"), "not a theme").unwrap();

            let toml_files = find_theme_files_in_directory(&temp_dir).unwrap();
            let json_files = find_json_files_in_directory(&temp_dir).unwrap();

            assert_eq!(toml_files.len(), 1);
            assert_eq!(json_files.len(), 1);

            // Cleanup
            fs::remove_dir_all(&temp_dir).unwrap();
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_non_windows_placeholder() {
        // This test just ensures the non-Windows version compiles
        assert!(true);
    }
}
