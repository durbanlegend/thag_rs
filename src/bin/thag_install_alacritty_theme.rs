/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto" }
dirs = "5.0"
*/

/// Install thag themes for Alacritty terminal emulator
///
/// This tool installs thag themes into Alacritty's configuration directory,
/// creates proper theme files with corrected color mappings, and updates
/// the Alacritty configuration file with appropriate import statements.
//# Purpose: Install and configure thag themes for Alacritty terminal
//# Categories: color, styling, terminal, theming, tools
use colored::Colorize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use thag_proc_macros::file_navigator;
use thag_styling::Theme;

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "🛠️  {} - Alacritty Theme Installer",
        "thag_install_alacritty_theme".bright_blue()
    );
    println!("{}", "=".repeat(70).dimmed());
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Get Alacritty directories
    let alacritty_config = get_alacritty_config_info()?;

    println!("📁 Alacritty configuration:");
    println!(
        "   Config directory: {}",
        alacritty_config
            .config_dir
            .display()
            .to_string()
            .bright_cyan()
    );
    println!(
        "   Themes directory: {}",
        alacritty_config
            .themes_dir
            .display()
            .to_string()
            .bright_cyan()
    );
    println!(
        "   Config file: {}",
        alacritty_config
            .config_file
            .display()
            .to_string()
            .bright_cyan()
    );
    println!();

    // Create directories if they don't exist
    fs::create_dir_all(&alacritty_config.themes_dir)?;

    // Select theme(s) to install
    let themes = select_themes(&mut navigator)?;

    if themes.is_empty() {
        println!("❌ No themes selected for installation.");
        return Ok(());
    }

    println!("🎨 Installing {} theme(s)...", themes.len());
    println!();

    let mut installed_themes = Vec::new();
    let mut installation_errors = Vec::new();

    // Install each theme
    for theme in &themes {
        match install_theme(theme, &alacritty_config) {
            Ok(theme_filename) => {
                installed_themes.push((theme.name.clone(), theme_filename));
                println!("   ✅ {}", theme.name.bright_green());
            }
            Err(e) => {
                let error_msg = e.to_string();
                installation_errors.push((theme.name.clone(), e));
                println!("   ❌ {}: {}", theme.name.bright_red(), error_msg.red());
            }
        }
    }

    // Update Alacritty configuration
    if !installed_themes.is_empty() {
        println!();
        match update_alacritty_config(&alacritty_config, &installed_themes) {
            Ok(()) => {
                println!("✅ Alacritty configuration updated successfully");
            }
            Err(e) => {
                println!(
                    "⚠️  Failed to update configuration: {}",
                    e.to_string().yellow()
                );
                show_manual_config_instructions(&installed_themes);
            }
        }
    }

    // Show summary and next steps
    show_installation_summary(&installed_themes, &installation_errors);
    show_verification_steps(&themes);

    println!("\n🎉 Theme installation completed!");
    Ok(())
}

#[derive(Debug)]
struct AlacrittyConfig {
    config_dir: PathBuf,
    themes_dir: PathBuf,
    config_file: PathBuf,
}

/// Get Alacritty configuration directories and files
fn get_alacritty_config_info() -> Result<AlacrittyConfig, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_dir = home_dir.join(".config/alacritty");
    let themes_dir = config_dir.join("themes");
    let config_file = config_dir.join("alacritty.toml");

    Ok(AlacrittyConfig {
        config_dir,
        themes_dir,
        config_file,
    })
}

/// Select themes to install using file navigator
fn select_themes(navigator: &mut FileNavigator) -> Result<Vec<Theme>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect, Select, Text};

    let selection_options = vec![
        "Select theme files (.toml)",
        "Select all themes from directory",
        "Install built-in theme by name",
        "Select from multiple built-in themes",
    ];

    let selection_method =
        Select::new("How would you like to select themes?", selection_options).prompt()?;

    match selection_method {
        "Select theme files (.toml)" => {
            let _extensions = vec!["toml", "TOML"];
            let mut selected_themes = Vec::new();

            loop {
                println!("\n📁 Select a theme file:");
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
        "Select all themes from directory" => {
            println!("\n📁 Select directory containing theme files:");
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

                    // Let user confirm which themes to install
                    if themes.len() > 1 {
                        let theme_names: Vec<String> = themes
                            .iter()
                            .map(|t| format!("{} - {}", t.name, t.description))
                            .collect();

                        let theme_names_len = theme_names.len();
                        let selected_names =
                            MultiSelect::new("Select themes to install:", theme_names.clone())
                                .with_default(&(0..theme_names_len).collect::<Vec<_>>())
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
        "Install built-in theme by name" => {
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
        "Select from multiple built-in themes" => {
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
                MultiSelect::new("Select themes to install:", theme_display_names.clone())
                    .prompt()?;

            let selected_themes = available_themes
                .into_iter()
                .enumerate()
                .filter(|(i, _)| selected_names.contains(&theme_display_names[*i]))
                .map(|(_, theme)| theme)
                .collect();

            Ok(selected_themes)
        }
        _ => Ok(vec![]),
    }
}

/// Find theme files in a directory
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

/// Install a single theme for Alacritty
fn install_theme(theme: &Theme, config: &AlacrittyConfig) -> Result<String, Box<dyn Error>> {
    // Generate Alacritty-compatible theme content
    let theme_content = generate_corrected_alacritty_theme(theme)?;

    // Create theme filename
    let theme_filename = format!("{}.toml", theme.name.replace(' ', "_").to_lowercase());
    let theme_path = config.themes_dir.join(&theme_filename);

    // Write theme file
    fs::write(&theme_path, &theme_content)
        .map_err(|e| format!("Failed to write theme file: {}", e))?;

    Ok(theme_filename)
}

/// Generate corrected Alacritty theme with proper color mappings
fn generate_corrected_alacritty_theme(theme: &Theme) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "# Alacritty Color Scheme: {}\n# Generated from thag theme with corrected mappings\n# {}\n\n",
        theme.name, theme.description
    ));

    let bg_color = theme.bg_rgbs.first().copied().unwrap_or((0, 0, 0));

    // Start colors section
    output.push_str("[colors]\n\n");

    // Primary colors
    output.push_str("[colors.primary]\n");
    output.push_str(&format!(
        "background = \"#{:02x}{:02x}{:02x}\"\n",
        bg_color.0, bg_color.1, bg_color.2
    ));

    if let Some(fg_color) = extract_rgb(&theme.palette.normal) {
        output.push_str(&format!(
            "foreground = \"#{:02x}{:02x}{:02x}\"\n",
            fg_color.0, fg_color.1, fg_color.2
        ));
    }

    if let Some(bright_fg) = extract_rgb(&theme.palette.emphasis) {
        output.push_str(&format!(
            "bright_foreground = \"#{:02x}{:02x}{:02x}\"\n",
            bright_fg.0, bright_fg.1, bright_fg.2
        ));
    }

    if let Some(dim_fg) = extract_rgb(&theme.palette.subtle) {
        output.push_str(&format!(
            "dim_foreground = \"#{:02x}{:02x}{:02x}\"\n",
            dim_fg.0, dim_fg.1, dim_fg.2
        ));
    }

    output.push_str("\n");

    // Normal colors (0-7) - CORRECTED MAPPINGS
    output.push_str("[colors.normal]\n");

    let normal_colors = [
        ("black", Some(bg_color)),                         // 0: background
        ("red", extract_rgb(&theme.palette.error)),        // 1: error
        ("green", extract_rgb(&theme.palette.success)),    // 2: success
        ("yellow", extract_rgb(&theme.palette.warning)),   // 3: warning
        ("blue", extract_rgb(&theme.palette.info)),        // 4: info
        ("magenta", extract_rgb(&theme.palette.heading1)), // 5: heading1
        ("cyan", extract_rgb(&theme.palette.heading3)),    // 6: heading3
        ("white", extract_rgb(&theme.palette.normal)),     // 7: normal text
    ];

    for (color_name, rgb_opt) in normal_colors {
        if let Some((r, g, b)) = rgb_opt {
            output.push_str(&format!(
                "{} = \"#{:02x}{:02x}{:02x}\"\n",
                color_name, r, g, b
            ));
        }
    }

    output.push_str("\n");

    // Bright colors (8-15) - CORRECTED MAPPINGS
    output.push_str("[colors.bright]\n");

    let bright_colors = [
        ("black", extract_rgb(&theme.palette.subtle)), // 8: subtle
        ("red", extract_rgb(&theme.palette.trace)),    // 9: trace (bright red)
        ("green", extract_rgb(&theme.palette.debug)),  // 10: debug (bright green)
        ("yellow", extract_rgb(&theme.palette.heading3)), // 11: heading3 (bright yellow)
        ("blue", extract_rgb(&theme.palette.heading2)), // 12: heading2 (bright blue)
        ("magenta", extract_rgb(&theme.palette.heading1)), // 13: heading1 (bright magenta)
        ("cyan", extract_rgb(&theme.palette.hint)),    // 14: hint (bright cyan)
        ("white", extract_rgb(&theme.palette.emphasis)), // 15: emphasis
    ];

    for (color_name, rgb_opt) in bright_colors {
        if let Some((r, g, b)) = rgb_opt {
            output.push_str(&format!(
                "{} = \"#{:02x}{:02x}{:02x}\"\n",
                color_name, r, g, b
            ));
        }
    }

    output.push_str("\n");

    // Cursor colors
    output.push_str("[colors.cursor]\n");
    if let Some(cursor_color) = extract_rgb(&theme.palette.emphasis) {
        output.push_str(&format!(
            "cursor = \"#{:02x}{:02x}{:02x}\"\n",
            cursor_color.0, cursor_color.1, cursor_color.2
        ));
        let cursor_text = if is_light_color(cursor_color) {
            (0, 0, 0)
        } else {
            (255, 255, 255)
        };
        output.push_str(&format!(
            "text = \"#{:02x}{:02x}{:02x}\"\n",
            cursor_text.0, cursor_text.1, cursor_text.2
        ));
    }

    output.push_str("\n");

    // Selection colors
    output.push_str("[colors.selection]\n");
    let selection_bg = adjust_brightness(bg_color, 1.4);
    output.push_str(&format!(
        "background = \"#{:02x}{:02x}{:02x}\"\n",
        selection_bg.0, selection_bg.1, selection_bg.2
    ));

    if let Some(selection_fg) = extract_rgb(&theme.palette.normal) {
        output.push_str(&format!(
            "text = \"#{:02x}{:02x}{:02x}\"\n",
            selection_fg.0, selection_fg.1, selection_fg.2
        ));
    }

    Ok(output)
}

/// Update Alacritty configuration with theme imports
fn update_alacritty_config(
    config: &AlacrittyConfig,
    installed_themes: &[(String, String)],
) -> Result<(), Box<dyn Error>> {
    use inquire::{Confirm, Select};

    if config.config_file.exists() {
        let existing_config = fs::read_to_string(&config.config_file)?;

        // Check if import statements already exist
        if existing_config.contains("import = [") {
            println!("⚠️  Configuration already contains import statements.");
            let update_anyway = Confirm::new("Update configuration anyway?")
                .with_default(false)
                .prompt()?;

            if !update_anyway {
                return Ok(());
            }
        }
    }

    // Let user choose which theme to set as active
    if installed_themes.len() == 1 {
        let (theme_name, theme_filename) = &installed_themes[0];
        update_config_with_theme(config, theme_filename)?;
        println!("✅ Set {} as active theme", theme_name.bright_cyan());
    } else {
        let theme_options: Vec<String> = installed_themes
            .iter()
            .map(|(name, _)| name.clone())
            .collect();

        let choice_options = vec![
            "Select active theme",
            "Don't set active theme (manual setup)",
        ];
        let setup_choice = Select::new("Configuration setup:", choice_options).prompt()?;

        if setup_choice == "Select active theme" {
            let selected_theme = Select::new("Choose active theme:", theme_options).prompt()?;

            if let Some((_, theme_filename)) = installed_themes
                .iter()
                .find(|(name, _)| name == &selected_theme)
            {
                update_config_with_theme(config, theme_filename)?;
                println!("✅ Set {} as active theme", selected_theme.bright_cyan());
            }
        }
    }

    Ok(())
}

/// Update configuration file with specific theme
fn update_config_with_theme(
    config: &AlacrittyConfig,
    theme_filename: &str,
) -> Result<(), Box<dyn Error>> {
    let import_line = format!("import = [\"themes/{}\"]\n", theme_filename);

    if config.config_file.exists() {
        let existing_config = fs::read_to_string(&config.config_file)?;

        // Simple approach: prepend the import to existing config
        let new_config = format!("[general]\n{}\n{}", import_line, existing_config);
        fs::write(&config.config_file, new_config)?;
    } else {
        // Create new config
        let new_config = format!(
            "# Alacritty Configuration\n# Generated by thag theme installer\n\n[general]\n{}\n",
            import_line
        );
        fs::write(&config.config_file, new_config)?;
    }

    Ok(())
}

/// Show manual configuration instructions
fn show_manual_config_instructions(installed_themes: &[(String, String)]) {
    println!("\n📖 {} Configuration:", "Manual".bright_blue());
    println!("Add the following to your alacritty.toml:");
    println!();
    println!("[general]");
    for (_, theme_filename) in installed_themes {
        println!("import = [\"themes/{}\"]", theme_filename);
    }
}

/// Show installation summary
fn show_installation_summary(
    installed_themes: &[(String, String)],
    errors: &[(String, Box<dyn Error>)],
) {
    println!();
    println!("📊 {} Summary:", "Installation".bright_blue());
    println!(
        "   Successfully installed: {}",
        installed_themes.len().to_string().bright_green()
    );
    println!(
        "   Failed installations: {}",
        errors.len().to_string().bright_red()
    );

    if !installed_themes.is_empty() {
        println!("\n✅ {} Themes:", "Installed".bright_green());
        for (theme_name, theme_filename) in installed_themes {
            println!(
                "   • {} ({})",
                theme_name.bright_cyan(),
                theme_filename.dimmed()
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

/// Show verification steps
fn show_verification_steps(themes: &[Theme]) {
    println!("\n🔍 {} Steps:", "Verification".bright_blue());
    println!("1. Restart Alacritty");
    println!("2. Check that colors match the expected theme");
    println!(
        "3. Run: {} (to check for config errors)",
        "alacritty --print-events".bright_cyan()
    );

    if !themes.is_empty() {
        let theme = &themes[0];
        println!(
            "\n💡 {} Colors for {}:",
            "Expected".bright_yellow(),
            theme.name.bright_cyan()
        );
        if let Some((r, g, b)) = theme.bg_rgbs.first() {
            println!("   Background: RGB({}, {}, {})", r, g, b);
        }
        if let Some((r, g, b)) = extract_rgb(&theme.palette.normal) {
            println!("   Normal text: RGB({}, {}, {})", r, g, b);
        }
    }
}

/// Extract RGB from style
fn extract_rgb(style: &thag_styling::Style) -> Option<(u8, u8, u8)> {
    style
        .foreground
        .as_ref()
        .and_then(|color_info| match &color_info.value {
            thag_styling::ColorValue::TrueColor { rgb } => Some((rgb[0], rgb[1], rgb[2])),
            _ => None,
        })
}

/// Adjust brightness of color
fn adjust_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    (
        ((r as f32 * factor).min(255.0).max(0.0)) as u8,
        ((g as f32 * factor).min(255.0).max(0.0)) as u8,
        ((b as f32 * factor).min(255.0).max(0.0)) as u8,
    )
}

/// Check if color is light
fn is_light_color((r, g, b): (u8, u8, u8)) -> bool {
    let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    luminance > 128.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use thag_styling::{ColorInfo, ColorSupport, Palette, Style, TermBgLuma};

    fn create_test_theme() -> Theme {
        let mut palette = Palette::default();
        palette.normal = Style::fg(ColorInfo::rgb(220, 220, 220));
        palette.error = Style::fg(ColorInfo::rgb(255, 100, 100));

        Theme {
            name: "Test Alacritty Theme".to_string(),
            filename: PathBuf::from("test_alacritty_theme.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette,
            backgrounds: vec!["#2a2a2a".to_string()],
            bg_rgbs: vec![(42, 42, 42)],
            description: "Test theme for Alacritty installation".to_string(),
        }
    }

    #[test]
    fn test_extract_rgb() {
        let style = Style::fg(ColorInfo::rgb(255, 128, 64));
        let rgb = extract_rgb(&style);
        assert_eq!(rgb, Some((255, 128, 64)));
    }

    #[test]
    fn test_adjust_brightness() {
        let original = (100, 150, 200);
        let brightened = adjust_brightness(original, 1.5);

        assert!(brightened.0 >= original.0);
        assert!(brightened.1 >= original.1);
        assert!(brightened.2 >= original.2);
    }

    #[test]
    fn test_is_light_color() {
        assert!(is_light_color((255, 255, 255)));
        assert!(!is_light_color((0, 0, 0)));
        assert!(is_light_color((200, 200, 200)));
        assert!(!is_light_color((50, 50, 50)));
    }

    #[test]
    fn test_generate_alacritty_theme() {
        let theme = create_test_theme();
        let result = generate_corrected_alacritty_theme(&theme);

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("[colors]"));
        assert!(content.contains("[colors.primary]"));
        assert!(content.contains("[colors.normal]"));
        assert!(content.contains("[colors.bright]"));
    }

    #[test]
    fn test_theme_file_discovery() {
        let temp_dir = std::env::temp_dir().join("thag_test_alacritty_themes");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        fs::write(temp_dir.join("theme1.toml"), "test theme 1").unwrap();
        fs::write(temp_dir.join("theme2.toml"), "test theme 2").unwrap();
        fs::write(temp_dir.join("not_theme.txt"), "not a theme").unwrap();

        let found_files = find_theme_files_in_directory(&temp_dir).unwrap();
        assert_eq!(found_files.len(), 2);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
