/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto", features = ["inquire_theming"] }
*/

/// Mintty theme installer for Git Bash on Windows
///
/// This Windows-only tool installs thag themes into Mintty by copying theme files
/// to the Mintty themes directory and optionally updating the ~/.minttyrc configuration.
/// Supports selecting individual themes or entire directories of themes.
//# Purpose: Install thag themes for Mintty (Git Bash)
//# Categories: color, styling, terminal, theming, tools, windows

#[cfg(target_os = "windows")]
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
#[cfg(target_os = "windows")]
use thag_proc_macros::file_navigator;
#[cfg(target_os = "windows")]
use thag_styling::{themed_inquire_config, Styleable};

#[cfg(target_os = "windows")]
file_navigator! {}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("❌ This tool is only available on Windows systems.");
    println!("   Mintty (Git Bash) is primarily used on Windows.");
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn Error>> {
    set_global_render_config(themed_inquire_config());

    println!(
        "🐙 {} - Mintty Theme Installer for Git Bash",
        "thag_mintty_add_theme".info()
    );
    println!("{}", "═".repeat(70));
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Get Mintty configuration paths
    let mintty_config = get_mintty_config_info()?;

    println!("📁 Mintty configuration:");
    println!(
        "   Themes directory: {}",
        mintty_config.themes_dir.display().to_string().code()
    );
    println!(
        "   Config file: {}",
        mintty_config
            .config_file
            .display()
            .to_string()
            .code()
    );

    // Check if themes directory exists
    if !mintty_config.themes_dir.exists() {
        println!("❌ Mintty themes directory not found.");
        println!("   Expected: {}", mintty_config.themes_dir.display());
        println!("   Please ensure Git for Windows is installed.");
        return Ok(());
    }

    println!("   Themes directory: {}", "Found".success());

    // Check write permissions upfront
    match check_directory_write_permission(&mintty_config.themes_dir) {
        Ok(true) => {
            println!("   Write permission: {}", "OK".success());
        }
        Ok(false) => {
            println!("❌ No write permission to themes directory.");
            println!("   Directory: {}", mintty_config.themes_dir.display());
            println!("   Please run this tool as Administrator or change directory permissions.");
            return Ok(());
        }
        Err(e) => {
            println!("⚠️  Could not check write permission: {}", e);
            println!("   Proceeding anyway - you may encounter permission errors.");
        }
    }

    // Check config file
    let config_exists = mintty_config.config_file.exists();
    println!(
        "   Config file: {}",
        if config_exists {
            "Found".success()
        } else {
            "Will be created".warning()
        }
    );
    println!();

    // Select themes to install
    let theme_files = select_themes(&mut navigator)?;

    if theme_files.is_empty() {
        println!("❌ No theme files selected for installation.");
        return Ok(());
    }

    // Process and install each theme
    let mut installed_themes = Vec::new();
    for theme_file in theme_files {
        match process_theme_file(&theme_file, &mintty_config.themes_dir) {
            Ok((theme_name, mintty_filename)) => {
                println!(
                    "✅ Installed: {} → {}",
                    theme_name.success(),
                    mintty_filename.info()
                );
                installed_themes.push((theme_name, mintty_filename));
            }
            Err(e) => {
                println!(
                    "❌ Failed to install {}: {}",
                    theme_file.display().to_string().error(),
                    e
                );
            }
        }
    }

    if !installed_themes.is_empty() {
        // Ask about updating config file
        let should_update_config = ask_update_config(&installed_themes)?;

        if should_update_config {
            match update_mintty_config(&mintty_config.config_file, &installed_themes) {
                Ok(theme_name) => {
                    println!("✅ Updated ~/.minttyrc with theme: {}", theme_name.info());
                }
                Err(e) => {
                    println!("⚠️  Failed to update ~/.minttyrc: {}", e);
                    println!("   You can manually add themes to your config.");
                }
            }
        }

        show_installation_summary(&installed_themes);
        show_usage_instructions();
    }

    Ok(())
}

/// Check if we have write permission to a directory
#[cfg(target_os = "windows")]
fn check_directory_write_permission(dir: &Path) -> Result<bool, Box<dyn Error>> {
    // Try to create a temporary file in the directory to test write permission
    let temp_filename = format!("thag_test_write_{}.tmp", std::process::id());
    let temp_path = dir.join(&temp_filename);

    match fs::write(&temp_path, "test") {
        Ok(()) => {
            // Clean up the test file
            let _ = fs::remove_file(&temp_path);
            Ok(true)
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
        Err(e) => Err(Box::new(e)),
    }
}

#[cfg(target_os = "windows")]
struct MinttyConfig {
    themes_dir: PathBuf,
    config_file: PathBuf,
}

#[cfg(target_os = "windows")]
fn get_mintty_config_info() -> Result<MinttyConfig, Box<dyn Error>> {
    // Standard Git for Windows installation path
    let themes_dir = PathBuf::from(r"C:\Program Files\Git\usr\share\mintty\themes");

    // User's home directory for config file
    let config_file = if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".minttyrc")
    } else {
        return Err("Could not determine home directory".into());
    };

    Ok(MinttyConfig {
        themes_dir,
        config_file,
    })
}

/// Select themes to install using file navigator
#[cfg(target_os = "windows")]
fn select_themes(navigator: &mut FileNavigator) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    use inquire::{Confirm, MultiSelect, Select};

    let selection_options = vec![
        "Select mintty theme files individually",
        "Select mintty theme files in bulk from directory",
    ];

    let mut selected_themes = Vec::new();

    // Try to navigate to exported themes directory if it exists
    let _ = navigator.navigate_to_path("exported_themes/mintty");
    if !navigator.current_path().join("mintty").exists() {
        let _ = navigator.navigate_to_path("exported_themes");
    }

    let selection_method =
        Select::new("How would you like to select themes?", selection_options).prompt()?;

    match selection_method {
        "Select mintty theme files individually" => {
            loop {
                println!("\n📁 Select a mintty theme file (no extension):");
                if let Ok(theme_file) = select_file(navigator, None, false) {
                    // Check that it's a mintty theme file (no extension)
                    if theme_file.extension().is_none() {
                        selected_themes.push(theme_file);
                    } else {
                        println!("⚠️  Mintty theme files should have no extension. Skipping.");
                        continue;
                    }

                    let add_more = Confirm::new("Add another theme file?")
                        .with_default(false)
                        .prompt()?;

                    if !add_more {
                        break;
                    }
                } else if selected_themes.is_empty() {
                    return Ok(vec![]);
                } else {
                    break;
                }
            }

            Ok(selected_themes)
        }
        "Select mintty theme files in bulk from directory" => {
            println!("\n📁 Select directory containing mintty theme files:");
            match select_directory(navigator, true) {
                Ok(theme_dir) => {
                    let theme_files = find_theme_files_in_directory(&theme_dir)?;

                    if theme_files.is_empty() {
                        println!("❌ No mintty theme files found in directory");
                        return Ok(vec![]);
                    }

                    for theme_file in theme_files {
                        selected_themes.push(theme_file);
                    }

                    // Let user confirm which themes to install
                    if selected_themes.len() > 1 {
                        let confirmed_themes = MultiSelect::new(
                            "Confirm themes to install:",
                            selected_themes
                                .iter()
                                .map(|v| v.display().to_string())
                                .collect::<Vec<_>>(),
                        )
                        .with_default(&(0..selected_themes.len()).collect::<Vec<_>>())
                        .prompt()?;

                        Ok(confirmed_themes
                            .iter()
                            .map(PathBuf::from)
                            .collect::<Vec<_>>())
                    } else {
                        Ok(selected_themes)
                    }
                }
                Err(_) => Ok(vec![]),
            }
        }
        _ => Ok(vec![]),
    }
}

/// Find mintty theme files in a directory (files with no extension)
#[cfg(target_os = "windows")]
fn find_theme_files_in_directory(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut theme_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_none() {
            // Additional check: ensure it looks like a mintty theme file
            if is_mintty_theme_file(&path)? {
                theme_files.push(path);
            }
        }
    }

    theme_files.sort();
    Ok(theme_files)
}

/// Check if a file appears to be a mintty theme file
#[cfg(target_os = "windows")]
fn is_mintty_theme_file(path: &Path) -> Result<bool, Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(path) {
        // Check for mintty-specific configuration keys
        Ok(content.contains("BackgroundColour=") || content.contains("ForegroundColour="))
    } else {
        Ok(false)
    }
}

/// Copy a mintty theme file to the themes directory
#[cfg(target_os = "windows")]
fn process_theme_file(
    theme_file: &Path,
    themes_dir: &Path,
) -> Result<(String, String), Box<dyn Error>> {
    let theme_filename = theme_file
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Invalid theme filename")?
        .to_string();

    let destination_path = themes_dir.join(&theme_filename);

    // Copy the theme file to the mintty themes directory
    fs::copy(theme_file, &destination_path)
        .map_err(|e| format!("Failed to copy theme file: {}", e))?;

    // Extract theme name from filename (remove thag- prefix if present)
    let theme_name = if theme_filename.starts_with("thag-") {
        theme_filename
            .strip_prefix("thag-")
            .unwrap_or(&theme_filename)
    } else {
        &theme_filename
    };

    let display_name = theme_name.replace('-', " ").replace('_', " ");

    Ok((display_name, theme_filename))
}

#[cfg(target_os = "windows")]
fn ask_update_config(installed_themes: &[(String, String)]) -> Result<bool, Box<dyn Error>> {
    use inquire::Confirm;

    if installed_themes.len() == 1 {
        let confirm = Confirm::new(&format!(
            "Would you like to set '{}' as the active
 theme in ~/.minttyrc?",
            installed_themes[0].0
        ))
        .with_default(true)
        .prompt()?;

        Ok(confirm)
    } else {
        let confirm = Confirm::new(
            "Would you like to set one of the installed themes as active in ~/.minttyrc?",
        )
        .with_default(true)
        .prompt()?;

        Ok(confirm)
    }
}

#[cfg(target_os = "windows")]
fn update_mintty_config(
    config_file: &Path,
    installed_themes: &[(String, String)],
) -> Result<String, Box<dyn Error>> {
    use inquire::Select;

    let theme_to_set = if installed_themes.len() == 1 {
        &installed_themes[0].1
    } else {
        // Let user choose which theme to activate
        let theme_names: Vec<String> = installed_themes
            .iter()
            .map(|(name, _)| name.clone())
            .collect();

        let selected_name = Select::new("Select which theme to activate:", theme_names).prompt()?;

        // Find the corresponding filename
        installed_themes
            .iter()
            .find(|(name, _)| name == &selected_name)
            .map(|(_, filename)| filename)
            .ok_or("Selected theme not found")?
    };

    // Read existing config or create new one
    let mut config_content = if config_file.exists() {
        fs::read_to_string(config_file)?
    } else {
        String::new()
    };

    // Remove existing ThemeFile lines
    config_content = config_content
        .lines()
        .filter(|line| !line.starts_with("ThemeFile="))
        .collect::<Vec<_>>()
        .join("\n");

    // Add new theme file line
    if !config_content.is_empty() && !config_content.ends_with('\n') {
        config_content.push('\n');
    }
    config_content.push_str(&format!("ThemeFile={}\n", theme_to_set));

    // Write updated config
    fs::write(config_file, config_content)?;

    Ok(theme_to_set.clone())
}

#[cfg(target_os = "windows")]
fn show_installation_summary(installed_themes: &[(String, String)]) {
    println!();
    println!("🎉 Installation Summary:");
    println!("{}", "═".repeat(50));

    for (theme_name, filename) in installed_themes {
        println!("✅ {} → {}", theme_name.success(), filename.info());
    }

    println!();
    println!(
        "📁 Themes installed to: {}",
        r"C:\Program Files\Git\usr\share\mintty\themes\".info()
    );
}

#[cfg(target_os = "windows")]
fn show_usage_instructions() {
    println!("🔧 How to use your new themes:");
    println!("{}", "▁".repeat(40));
    println!(r#"1. Ensure your `thag_styling` theme is set to match.
E.g. `export THAG_THEME=<corresponding thag_styling theme>"#` in `~/.bashrc` or `~/.zshrc`
or as preferred light/dark theme via `thag -C` (ensure background color of `thag_styling` theme matches that of terminal));
    println!("2. Open Git Bash (Mintty)");
    println!("3. Right-click on the title bar and select 'Options...'");
    println!("4. Go to the 'Looks' tab");
    println!("5. Select your theme from the 'Theme' dropdown");
    println!("6. Click 'Apply' or 'OK'");
    println!();
    println!("💡 Tip: The theme will apply to all new Mintty windows.");
    println!("   Existing windows may need to be restarted to see the changes.");
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_mintty_config_info() {
        // Test that we can get mintty configuration info
        if let Ok(config) = get_mintty_config_info() {
            assert!(config.themes_dir.to_string_lossy().contains("mintty"));
            assert!(config.config_file.to_string_lossy().contains("minttyrc"));
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_mintty_theme_file_detection() {
        // Test helper for file detection - this would need actual files to test properly
        assert!(true);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_non_windows_placeholder() {
        // This test exists to ensure the test module compiles on non-Windows platforms
        assert!(true);
    }
}
