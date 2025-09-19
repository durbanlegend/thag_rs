/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Install generated themes for Alacritty terminal emulator
///
/// This tool installs Alacritty themes into Alacritty's configuration directory
/// and updates the Alacritty configuration file with appropriate import statements.
/// The themes will typically have been created by `thag_gen_terminal_themes.rs`.
//# Purpose: Install and configure thag themes for Alacritty terminal
//# Categories: color, styling, terminal, theming, tools
use inquire::set_global_render_config;
use std::clone::Clone;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::string::ToString;
use thag_proc_macros::file_navigator;
use thag_styling::{themed_inquire_config, Styleable};
use toml_edit::{DocumentMut, Item, Value};

file_navigator! {}

fn main() -> Result<(), Box<dyn Error>> {
    set_global_render_config(themed_inquire_config());

    println!(
        "🎨 {} - Alacritty Theme Installer",
        "thag_install_alacritty_theme".info()
    );
    println!("{}", "=".repeat(70));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Get Alacritty directories
    let alacritty_config = get_alacritty_config_info()?;

    println!("📁 Alacritty configuration:");
    println!(
        "   Config directory: {}",
        alacritty_config.config_dir.display().to_string().info()
    );
    println!(
        "   Themes directory: {}",
        alacritty_config.themes_dir.display().to_string().info()
    );
    println!(
        "   Config file: {}",
        alacritty_config.config_file.display().to_string().info()
    );
    println!();

    // Create directories if they don't exist
    fs::create_dir_all(&alacritty_config.themes_dir)?;

    // Select theme(s) to install
    let theme_paths = select_themes(&mut navigator)?;

    if theme_paths.is_empty() {
        println!("❌ No themes selected for installation.");
        return Ok(());
    }

    println!("🎨 Installing {} theme(s)...", theme_paths.len());
    println!();

    // Update Alacritty configuration
    println!();
    let theme_filenames: Vec<String> = theme_paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|f| f.to_str()))
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let mut installed_themes = Vec::new();
    let mut installation_errors: Vec<(String, Box<dyn Error>)> = Vec::new();
    let destination_path = alacritty_config.themes_dir.clone();
    for theme_source_path in &theme_paths {
        let theme_filename = theme_source_path
            .file_name()
            .ok_or("Can't extract filename")?
            .to_string_lossy();
        // Attempt to copy the file
        // let result = copy_theme(theme_source_path, &destination_path);
        let result = fs::copy(
            theme_source_path,
            destination_path.join(theme_filename.to_string()),
        );
        match result {
            Ok(_) => {
                installed_themes.push(theme_filename.clone());
                println!("   ✅ {}", theme_filename.to_string().success());
            }
            Err(e) => {
                let e_str = e.to_string();
                installation_errors.push((theme_filename.to_string(), Box::new(e)));
                println!(
                    "   ❌ {}: {}",
                    theme_filename.to_string().error(),
                    e_str.error()
                );
            }
        }
    }

    match update_alacritty_config(
        &alacritty_config,
        &installed_themes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>(),
    ) {
        Ok(()) => {
            println!("✅ Alacritty configuration updated successfully");
        }
        Err(e) => {
            println!(
                "⚠️  Failed to update configuration: {}",
                e.to_string().warning()
            );
            show_manual_config_instructions(&theme_filenames);
        }
    }

    // Show summary and next steps
    show_installation_summary(
        &installed_themes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>(),
        &installation_errors,
    );
    show_verification_steps(&theme_filenames);

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
fn select_themes(navigator: &mut FileNavigator) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect, Select};

    // let opt_indiv = "Select theme files (.toml) individually";
    // let opt_bulk = "Select theme files in bulk from directory";
    let selection_options = vec![
        "Select theme files (.toml) individually",
        "Select theme files in bulk from directory",
    ];

    let mut selected_themes = Vec::new();

    // Make an attempt to find the most likely path
    let _ = navigator.navigate_to_path("exported_themes/alacritty");

    let selection_method =
        Select::new("How would you like to select themes?", selection_options).prompt()?;

    match selection_method {
        "Select theme files (.toml) individually" => {
            let extensions = "toml,TOML";

            loop {
                println!("\n📁 Select a theme file:");
                if let Ok(theme_file) = select_file(navigator, Some(extensions), false) {
                    selected_themes.push(theme_file);

                    let add_more = Confirm::new("Add another theme file?")
                        .with_default(false)
                        .prompt()?;

                    if !add_more {
                        break;
                    }
                } else {
                    if selected_themes.is_empty() {
                        return Ok(vec![]);
                    }
                    break;
                }
            }

            Ok(selected_themes)
        }
        "Select theme files in bulk from directory" => {
            println!("\n📁 Select directory containing theme files:");
            match select_directory(navigator, true) {
                Ok(theme_dir) => {
                    let theme_files = find_theme_files_in_directory(&theme_dir)?;

                    if theme_files.is_empty() {
                        println!("❌ No .toml theme files found in directory");
                        return Ok(vec![]);
                    }

                    // let mut themes = Vec::new();
                    for theme_file in theme_files {
                        selected_themes.push(theme_file);
                    }

                    // Let user confirm which themes to install
                    if selected_themes.len() > 1 {
                        let selected_themes = MultiSelect::new(
                            "Confirm themes to install:",
                            selected_themes // .clone()
                                .iter()
                                .map(|v| v.display().to_string())
                                .collect::<Vec<_>>(),
                        )
                        .with_default(&(0..selected_themes.len()).collect::<Vec<_>>())
                        .prompt()?;

                        Ok(selected_themes
                            .iter()
                            .map(PathBuf::from)
                            .collect::<Vec<_>>())
                    } else {
                        Ok(vec![])
                    }
                }
                Err(_) => Ok(vec![]),
            }
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

/// Update Alacritty configuration with theme imports
fn update_alacritty_config(
    config: &AlacrittyConfig,
    installed_themes: &[String],
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
        let theme_filename = &installed_themes[0];
        update_config_with_theme(config, theme_filename)?;
        println!("✅ Set {} as active theme", theme_filename.info());
    } else {
        let theme_options: Vec<String> = installed_themes.to_vec();

        let choice_options = vec![
            "Select active theme",
            "Don't set active theme (manual setup)",
        ];
        let setup_choice = Select::new("Configuration setup:", choice_options).prompt()?;

        for theme_filename in installed_themes {
            update_config_with_theme(config, theme_filename)?;
        }
        if setup_choice == "Select active theme" {
            let selected_theme = Select::new("Choose active theme:", theme_options).prompt()?;

            if let Some(theme_filename) = installed_themes
                .iter()
                .find(|&name| name == &selected_theme)
            {
                update_config_with_theme(config, theme_filename)?;
                println!("✅ Set {} as active theme", selected_theme.info());
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
    let import_line = format!("themes/{theme_filename}",);

    if config.config_file.exists() {
        let existing_config = fs::read_to_string(&config.config_file)?;

        // Parse as a mutable TOML document
        let doc = &mut existing_config.parse::<DocumentMut>()?;

        // Navigate to general.import
        if let Some(imports) = doc["general"]["import"].as_array_mut() {
            // Remove duplicates of new_value
            imports.retain(|item| item.as_str() != Some(&import_line));

            // Push the new value at the end
            imports.push(Value::from(&import_line));
        } else {
            // If the array doesn't exist, create it
            let mut arr = toml_edit::Array::default();
            arr.push(Value::from(&import_line));
            doc["general"]["import"] = Item::Value(Value::Array(arr));
        }

        // Write back the modified TOML
        fs::write(&config.config_file, doc.to_string())?;

        println!(
            "✅ Updated {}, moved {import_line} to the end of [general.import]",
            config.config_file.display()
        );
    } else {
        // Create new config
        let new_config = format!(
            "# Alacritty Configuration\n# Generated by thag theme installer\n\n[general]\n{import_line}\n"
        );
        fs::write(&config.config_file, new_config)?;
    }

    Ok(())
}

/// Show manual configuration instructions
fn show_manual_config_instructions(installed_themes: &[String]) {
    println!("\n📖 {} Configuration:", "Manual".info());
    println!("Add the following to your alacritty.toml:");
    println!();
    println!("[general]");
    for theme_filename in installed_themes {
        println!("import = [\"themes/{theme_filename}\"]");
    }
}

/// Show installation summary
fn show_installation_summary(installed_themes: &[String], errors: &[(String, Box<dyn Error>)]) {
    println!();
    println!("📊 {} Summary:", "Installation".info());
    println!(
        "   Successfully installed: {}",
        installed_themes.len().to_string().success()
    );
    println!(
        "   Failed installations: {}",
        errors.len().to_string().error()
    );

    if !installed_themes.is_empty() {
        println!("\n✅ {} Themes:", "Installed".success());
        for theme_name in installed_themes {
            println!("   • {})", theme_name.info(),);
        }
    }

    if !errors.is_empty() {
        println!("\n❌ {} Failures:", "Installation".error());
        for (theme_name, error) in errors {
            println!("   • {}: {}", theme_name, error.to_string().error());
        }
    }
}

/// Show verification steps
fn show_verification_steps(_installed_themes: &[String]) {
    println!("\n🔍 {} Steps:", "Verification".info());
    println!(r#"1. Ensure your `thag_styling` theme is set to match.
E.g. `export THAG_THEME=<corresponding thag_styling theme>"#` in `~/.bashrc` or `~/.zshrc`
or as preferred light/dark theme via `thag -C` (ensure background color of `thag_styling` theme matches that of terminal));
    println!("2. Restart Alacritty if necessary");
    println!("3. Check that colors match the expected theme");
    println!(
        "34. Run: {} (to check for config errors)",
        "alacritty --print-events".info()
    );
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
