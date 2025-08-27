/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["config", "simplelog"] }
thag_styling = { version = "0.2, thag-auto" }
dirs = "5.0"
*/

/// Mintty theme installer for Git Bash on Windows
///
/// This Windows-only tool installs thag themes into Mintty by copying theme files
/// to the Mintty themes directory and optionally updating the ~/.minttyrc configuration.
/// Supports selecting individual themes or entire directories of themes.
//# Purpose: Install thag themes for Mintty (Git Bash)
//# Categories: color, styling, terminal, theming, tools, windows
#[cfg(target_os = "windows")]
use colored::Colorize;
#[cfg(target_os = "windows")]
use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
#[cfg(target_os = "windows")]
use thag_proc_macros::file_navigator;
#[cfg(target_os = "windows")]
use thag_rs::Theme;
#[cfg(target_os = "windows")]
use thag_styling::exporters::{ExportFormat, ThemeExporter};

#[cfg(target_os = "windows")]
file_navigator! {}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("❌ This tool is only available on Windows systems.");
    println!("   Mintty (Git Bash) is primarily used on Windows.");
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "🐙 {} - Mintty Theme Installer for Git Bash",
        "thag_mintty_add_theme".bright_blue()
    );
    println!("{}", "=".repeat(70).dimmed());
    println!();

    // Initialize file navigator
    let mut navigator = FileNavigator::new();

    // Get Mintty paths
    let mintty_config = get_mintty_config_info()?;

    println!("📁 Mintty configuration:");
    println!(
        "   Themes directory: {}",
        mintty_config.themes_dir.display().to_string().bright_cyan()
    );
    println!(
        "   Config file: {}",
        mintty_config
            .config_file
            .display()
            .to_string()
            .bright_cyan()
    );

    // Check if themes directory exists
    if !mintty_config.themes_dir.exists() {
        println!("❌ Mintty themes directory not found.");
        println!("   Expected: {}", mintty_config.themes_dir.display());
        println!("   Please ensure Git for Windows is installed.");
        return Ok(());
    }

    println!("   Themes directory: {}", "Found".bright_green());

    // Check config file
    let config_exists = mintty_config.config_file.exists();
    println!(
        "   Config file: {}",
        if config_exists {
            "Found".bright_green()
        } else {
            "Will be created".bright_yellow()
        }
    );
    println!();

    // Select themes to install
    let themes = select_themes_for_installation(&mut navigator)?;

    if themes.is_empty() {
        println!("❌ No themes selected for installation.");
        return Ok(());
    }

    println!("🎨 Installing {} theme(s)...", themes.len());
    println!();

    // Install each theme
    let mut installed_themes = Vec::new();
    for theme in &themes {
        match install_mintty_theme(theme, &mintty_config.themes_dir) {
            Ok(theme_filename) => {
                println!("✅ Installed: {}", theme.name.bright_green());
                installed_themes.push((theme.name.clone(), theme_filename));
            }
            Err(e) => {
                println!("❌ Failed to install {}: {}", theme.name.bright_red(), e);
            }
        }
    }

    if !installed_themes.is_empty() {
        // Ask about updating config file
        let should_update_config = ask_update_config(&installed_themes)?;

        if should_update_config {
            match update_mintty_config(&mintty_config.config_file, &installed_themes) {
                Ok(()) => {
                    println!("✅ Updated ~/.minttyrc configuration");
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

#[cfg(target_os = "windows")]
fn select_themes_for_installation(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    println!("🎯 Select theme installation method:");
    println!("   1. Select individual TOML theme files");
    println!("   2. Select a directory containing exported Mintty themes");
    println!("   3. Select from built-in themes by name");
    println!("   4. Select multiple built-in themes");
    println!();

    print!("Enter your choice (1-4): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => select_individual_toml_themes(navigator),
        "2" => select_exported_mintty_themes(navigator),
        "3" => select_builtin_theme_by_name(),
        "4" => select_multiple_builtin_themes(),
        _ => {
            println!("❌ Invalid choice. Please run the tool again.");
            Ok(vec![])
        }
    }
}

#[cfg(target_os = "windows")]
fn select_individual_toml_themes(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    println!("📂 Navigate to select TOML theme files...");

    let theme_files = navigator.select_files("Select TOML theme files", |path| {
        path.extension().map_or(false, |ext| ext == "toml")
    })?;

    let mut themes = Vec::new();
    for file_path in theme_files {
        match thag_styling::Theme::load_from_file(&file_path) {
            Ok(theme) => themes.push(theme),
            Err(e) => println!("⚠️  Failed to load {}: {}", file_path.display(), e),
        }
    }

    Ok(themes)
}

#[cfg(target_os = "windows")]
fn select_exported_mintty_themes(
    navigator: &mut FileNavigator,
) -> Result<Vec<Theme>, Box<dyn Error>> {
    println!("📂 Navigate to select a directory containing exported Mintty themes...");

    let dir_path = navigator.select_directory("Select directory with Mintty theme files")?;

    let theme_files = find_mintty_theme_files_in_directory(&dir_path)?;

    if theme_files.is_empty() {
        println!("❌ No Mintty theme files found in the selected directory.");
        return Ok(vec![]);
    }

    println!("📋 Found {} Mintty theme file(s):", theme_files.len());
    for (i, file) in theme_files.iter().enumerate() {
        println!("   {}. {}", i + 1, file.display());
    }

    // For exported Mintty themes, we need to create minimal theme objects
    // since they don't contain all the metadata
    let mut themes = Vec::new();
    for file_path in theme_files {
        let theme_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unnamed Theme")
            .replace("_mintty", "")
            .replace("_", " ");

        // Create a minimal theme object for exported themes
        let theme = create_theme_from_exported_file(&file_path, &theme_name)?;
        themes.push(theme);
    }

    Ok(themes)
}

#[cfg(target_os = "windows")]
fn select_builtin_theme_by_name() -> Result<Vec<Theme>, Box<dyn Error>> {
    println!("📝 Enter the name of a built-in theme:");
    print!("Theme name: ");
    io::stdout().flush()?;

    let mut theme_name = String::new();
    io::stdin().read_line(&mut theme_name)?;
    let theme_name = theme_name.trim();

    match thag_styling::Theme::load_builtin(theme_name) {
        Ok(theme) => Ok(vec![theme]),
        Err(_) => {
            println!("❌ Built-in theme '{}' not found.", theme_name);
            println!("   Try running 'thag list-themes' to see available themes.");
            Ok(vec![])
        }
    }
}

#[cfg(target_os = "windows")]
fn select_multiple_builtin_themes() -> Result<Vec<Theme>, Box<dyn Error>> {
    println!("📋 Available built-in themes:");
    let builtin_themes = thag_styling::Theme::list_builtin_themes();

    for (i, theme_name) in builtin_themes.iter().enumerate() {
        println!("   {}. {}", i + 1, theme_name);
    }

    println!();
    println!("📝 Enter theme numbers (comma-separated, e.g., 1,3,5) or 'all' for all themes:");
    print!("Selection: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let selected_themes = if input.eq_ignore_ascii_case("all") {
        builtin_themes
    } else {
        let indices: Result<Vec<usize>, _> = input
            .split(',')
            .map(|s| s.trim().parse::<usize>().map(|n| n.saturating_sub(1)))
            .collect();

        match indices {
            Ok(indices) => indices
                .into_iter()
                .filter(|&i| i < builtin_themes.len())
                .map(|i| builtin_themes[i].clone())
                .collect(),
            Err(_) => {
                println!("❌ Invalid selection format.");
                return Ok(vec![]);
            }
        }
    };

    let mut themes = Vec::new();
    for theme_name in selected_themes {
        match thag_styling::Theme::load_builtin(&theme_name) {
            Ok(theme) => themes.push(theme),
            Err(e) => println!("⚠️  Failed to load built-in theme '{}': {}", theme_name, e),
        }
    }

    Ok(themes)
}

#[cfg(target_os = "windows")]
fn find_mintty_theme_files_in_directory(dir_path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut theme_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // Look for files that end with "_mintty" or have no extension and contain mintty config
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                if filename.contains("mintty")
                    || (path.extension().is_none() && is_mintty_theme_file(&path)?)
                {
                    theme_files.push(path);
                }
            }
        }
    }

    theme_files.sort();
    Ok(theme_files)
}

#[cfg(target_os = "windows")]
fn is_mintty_theme_file(path: &Path) -> Result<bool, Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(path) {
        // Check for mintty-specific configuration keys
        Ok(content.contains("BackgroundColour=") || content.contains("ForegroundColour="))
    } else {
        Ok(false)
    }
}

#[cfg(target_os = "windows")]
fn create_theme_from_exported_file(
    file_path: &Path,
    theme_name: &str,
) -> Result<Theme, Box<dyn Error>> {
    // For exported themes, create a minimal theme object
    // We can't recreate the full palette from the exported format, but we can create a basic one
    use thag_styling::{ColorSupport, Palette, TermBgLuma};

    Ok(Theme {
        name: theme_name.to_string(),
        filename: file_path.to_path_buf(),
        is_builtin: false,
        term_bg_luma: TermBgLuma::Dark, // Default assumption
        min_color_support: ColorSupport::TrueColor,
        palette: Palette::default(),
        backgrounds: vec!["#1e1e2e".to_string()], // Default
        bg_rgbs: vec![(30, 30, 46)],              // Default
        description: "Exported Mintty theme".to_string(),
    })
}

#[cfg(target_os = "windows")]
fn install_mintty_theme(theme: &Theme, themes_dir: &Path) -> Result<String, Box<dyn Error>> {
    // Generate the mintty theme content
    let theme_content = ExportFormat::Mintty
        .export_theme(theme)
        .map_err(|e| format!("Failed to export theme: {}", e))?;

    // Create theme filename (mintty themes have no extension)
    let theme_filename = format!("thag-{}", theme.name.to_lowercase().replace(' ', "-"));
    let theme_path = themes_dir.join(&theme_filename);

    // Write theme file
    fs::write(&theme_path, theme_content)
        .map_err(|e| format!("Failed to write theme file: {}", e))?;

    Ok(theme_filename)
}

#[cfg(target_os = "windows")]
fn ask_update_config(installed_themes: &[(String, String)]) -> Result<bool, Box<dyn Error>> {
    if installed_themes.len() == 1 {
        println!(
            "❓ Would you like to set '{}' as the active theme in ~/.minttyrc? (y/n): ",
            installed_themes[0].0
        );
    } else {
        println!("❓ Would you like to set one of the installed themes as active in ~/.minttyrc? (y/n): ");
    }

    print!("Choice: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

#[cfg(target_os = "windows")]
fn update_mintty_config(
    config_file: &Path,
    installed_themes: &[(String, String)],
) -> Result<(), Box<dyn Error>> {
    let theme_to_set = if installed_themes.len() == 1 {
        &installed_themes[0].1
    } else {
        // Let user choose which theme to activate
        println!("📋 Select which theme to activate:");
        for (i, (name, _)) in installed_themes.iter().enumerate() {
            println!("   {}. {}", i + 1, name);
        }

        print!("Enter choice (1-{}): ", installed_themes.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let choice: usize = input.trim().parse().map_err(|_| "Invalid choice")?;

        if choice == 0 || choice > installed_themes.len() {
            return Err("Invalid theme selection".into());
        }

        &installed_themes[choice - 1].1
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

    Ok(())
}

#[cfg(target_os = "windows")]
fn show_installation_summary(installed_themes: &[(String, String)]) {
    println!();
    println!("🎉 Installation Summary:");
    println!("{}", "=".repeat(50).dimmed());

    for (theme_name, filename) in installed_themes {
        println!(
            "✅ {} → {}",
            theme_name.bright_green(),
            filename.bright_cyan()
        );
    }

    println!();
    println!(
        "📁 Themes installed to: {}",
        r"C:\Program Files\Git\usr\share\mintty\themes\".bright_cyan()
    );
}

#[cfg(target_os = "windows")]
fn show_usage_instructions() {
    println!("🔧 How to use your new themes:");
    println!("{}", "-".repeat(40).dimmed());
    println!("1. Open Git Bash (Mintty)");
    println!("2. Right-click on the title bar and select 'Options...'");
    println!("3. Go to the 'Looks' tab");
    println!("4. Select your theme from the 'Theme' dropdown");
    println!("5. Click 'Apply' or 'OK'");
    println!();
    println!("💡 Tip: The theme will apply to all new Mintty windows.");
    println!("   Existing windows may need to be restarted to see the changes.");
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use super::*;
    #[cfg(target_os = "windows")]
    use thag_styling::{ColorSupport, Palette, TermBgLuma};

    #[cfg(target_os = "windows")]
    fn create_test_theme() -> Theme {
        Theme {
            name: "Test Mintty Theme".to_string(),
            filename: PathBuf::from("test.toml"),
            is_builtin: false,
            term_bg_luma: TermBgLuma::Dark,
            min_color_support: ColorSupport::TrueColor,
            palette: Palette::default(),
            backgrounds: vec!["#1e1e2e".to_string()],
            bg_rgbs: vec![(30, 30, 46)],
            description: "A test theme for mintty installation".to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_theme_export() {
        let theme = create_test_theme();
        let result = ExportFormat::Mintty.export_theme(&theme);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("BackgroundColour=30,30,46"));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_non_windows_placeholder() {
        // This test exists to ensure the test module compiles on non-Windows platforms
        assert!(true);
    }
}
