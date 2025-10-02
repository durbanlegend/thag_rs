//! Interactive theme editor for manual color role adjustments
//!
//! This tool allows you to interactively edit theme color assignments,
//! particularly useful when automatic conversion doesn't quite match your preferences.
//!
//! # Purpose:
//! Edit and customize theme color role assignments interactively
//!
//! # Categories: theme, color, styling, interactive

/*[toml]
[dependencies]
thag_styling = { version = "0.2, thag-auto" }
inquire = "0.7"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
clap = { version = "4.5", features = ["derive"] }
*/

use clap::Parser;
use inquire::{Confirm, Select};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use thag_styling::{ColorValue, Palette, Role, Style, Theme};

#[derive(Parser, Debug)]
#[command(author, version, about = "Interactive theme editor", long_about = None)]
struct Cli {
    /// Input theme file (.toml format)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Output theme file (defaults to input file if not specified)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Create backup of original file
    #[arg(short, long, default_value_t = true)]
    backup: bool,
}

/// A color candidate with its hex value and current role assignment
#[derive(Clone, Debug)]
struct ColorCandidate {
    hex: String,
    rgb: [u8; 3],
    current_role: Option<Role>,
}

impl ColorCandidate {
    fn from_style(style: &Style, role: Option<Role>) -> Option<Self> {
        style.foreground.as_ref().and_then(|color_info| {
            if let ColorValue::TrueColor { rgb } = &color_info.value {
                Some(Self {
                    hex: format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]),
                    rgb: *rgb,
                    current_role: role,
                })
            } else {
                None
            }
        })
    }

    fn display_name(&self) -> String {
        if let Some(role) = &self.current_role {
            format!("{} (currently {:?})", self.hex, role)
        } else {
            self.hex.clone()
        }
    }

    fn preview(&self) -> String {
        let style = Style::with_rgb(self.rgb);
        format!("{} {}", style.paint("████"), self.display_name())
    }
}

/// Main theme editor state
struct ThemeEditor {
    theme: Theme,
    original_palette: Palette,
    modified: bool,
}

impl ThemeEditor {
    fn new(theme: Theme) -> Self {
        let original_palette = theme.palette.clone();
        Self {
            theme,
            original_palette,
            modified: false,
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("\n🎨 Theme Editor: {}\n", self.theme.name);
        self.show_theme_info();

        loop {
            let action = self.select_action()?;

            match action.as_str() {
                "Edit color role" => self.edit_role()?,
                "Swap two roles" => self.swap_roles()?,
                "Reset to original" => self.reset_palette()?,
                "Show current palette" => self.show_palette()?,
                "Save and exit" => {
                    if self.save_and_exit()? {
                        break;
                    }
                }
                "Exit without saving" => {
                    if self.confirm_exit()? {
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn show_theme_info(&self) {
        println!("📋 Theme: {}", self.theme.name);
        println!("🌓 Type: {:?}", self.theme.term_bg_luma);
        println!("🎨 Color Support: {:?}", self.theme.min_color_support);
        if let Some(bg) = self.theme.bg_rgbs.first() {
            println!("🖼️  Background: #{:02x}{:02x}{:02x}", bg.0, bg.1, bg.2);
        }
        println!();
    }

    fn select_action(&self) -> Result<String, Box<dyn Error>> {
        let modified_indicator = if self.modified { " [MODIFIED]" } else { "" };

        let actions = vec![
            "Edit color role",
            "Swap two roles",
            "Reset to original",
            "Show current palette",
            "Save and exit",
            "Exit without saving",
        ];

        let prompt = format!("What would you like to do?{}", modified_indicator);
        let selection = Select::new(&prompt, actions).prompt()?;

        Ok(selection.to_string())
    }

    fn edit_role(&mut self) -> Result<(), Box<dyn Error>> {
        // Select which role to edit
        let role = self.select_role("Which role would you like to edit?")?;

        // Get current color for this role
        let current_style = self.theme.style_for(role);
        let current_hex = Self::style_to_hex(&current_style);

        println!(
            "\nCurrent color for {:?}: {}",
            role,
            current_style.paint(format!("████ {}", current_hex))
        );

        // Get all available colors from the palette
        let candidates = self.collect_color_candidates(Some(role));

        if candidates.is_empty() {
            println!("❌ No color candidates available!");
            return Ok(());
        }

        let options: Vec<String> = candidates.iter().map(|c| c.preview()).collect();

        let selection = Select::new("Select new color:", options).prompt()?;

        // Find the selected candidate
        let selected = candidates
            .iter()
            .find(|c| c.preview() == selection)
            .ok_or("Color not found")?;

        // Update the role
        self.update_role(role, selected.rgb)?;
        self.modified = true;

        println!("✅ Updated {:?} to {}", role, selected.hex);

        Ok(())
    }

    fn swap_roles(&mut self) -> Result<(), Box<dyn Error>> {
        println!("\n🔄 Swap two role colors");

        let role1 = self.select_role("Select first role:")?;
        let role2 = self.select_role("Select second role:")?;

        if role1 == role2 {
            println!("⚠️  Cannot swap a role with itself!");
            return Ok(());
        }

        // Get current colors
        let style1 = self.theme.style_for(role1).clone();
        let style2 = self.theme.style_for(role2).clone();

        // Extract RGB values
        let rgb1 = Self::extract_rgb(&style1)?;
        let rgb2 = Self::extract_rgb(&style2)?;

        // Swap them
        self.update_role(role1, rgb2)?;
        self.update_role(role2, rgb1)?;
        self.modified = true;

        println!("✅ Swapped {:?} ↔ {:?}", role1, role2);

        Ok(())
    }

    fn reset_palette(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.modified {
            println!("ℹ️  Palette has not been modified.");
            return Ok(());
        }

        let confirm = Confirm::new("Reset all changes to original palette?")
            .with_default(false)
            .prompt()?;

        if confirm {
            self.theme.palette = self.original_palette.clone();
            self.modified = false;
            println!("✅ Palette reset to original");
        }

        Ok(())
    }

    fn show_palette(&self) -> Result<(), Box<dyn Error>> {
        println!("\n📊 Current Palette:\n");

        let roles = vec![
            Role::Heading1,
            Role::Heading2,
            Role::Heading3,
            Role::Error,
            Role::Warning,
            Role::Success,
            Role::Info,
            Role::Emphasis,
            Role::Code,
            Role::Normal,
            Role::Subtle,
            Role::Hint,
            Role::Debug,
            Role::Link,
            Role::Quote,
            Role::Commentary,
        ];

        for role in roles {
            let style = self.theme.style_for(role);
            let hex = Self::style_to_hex(&style);
            println!(
                "  {:12} │ {} {}",
                format!("{:?}", role),
                style.paint("████"),
                hex
            );
        }

        println!();
        Ok(())
    }

    fn save_and_exit(&self) -> Result<bool, Box<dyn Error>> {
        if !self.modified {
            println!("ℹ️  No changes to save.");
            return Ok(true);
        }

        let confirm = Confirm::new("Save changes?").with_default(true).prompt()?;

        if confirm {
            // Save will be handled by the caller
            println!("✅ Changes will be saved");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn confirm_exit(&self) -> Result<bool, Box<dyn Error>> {
        if self.modified {
            let confirm = Confirm::new("Exit without saving changes?")
                .with_default(false)
                .prompt()?;
            Ok(confirm)
        } else {
            Ok(true)
        }
    }

    fn select_role(&self, prompt: &str) -> Result<Role, Box<dyn Error>> {
        let roles = vec![
            ("Heading1", Role::Heading1),
            ("Heading2", Role::Heading2),
            ("Heading3", Role::Heading3),
            ("Error", Role::Error),
            ("Warning", Role::Warning),
            ("Success", Role::Success),
            ("Info", Role::Info),
            ("Emphasis", Role::Emphasis),
            ("Code", Role::Code),
            ("Normal", Role::Normal),
            ("Subtle", Role::Subtle),
            ("Hint", Role::Hint),
            ("Debug", Role::Debug),
            ("Link", Role::Link),
            ("Quote", Role::Quote),
            ("Commentary", Role::Commentary),
        ];

        let options: Vec<String> = roles
            .iter()
            .map(|(name, role)| {
                let style = self.theme.style_for(*role);
                let hex = Self::style_to_hex(&style);
                format!("{:12} {} {}", name, style.paint("██"), hex)
            })
            .collect();

        let selection = Select::new(prompt, options).prompt()?;

        // Extract role from selection
        let role_name = selection.split_whitespace().next().unwrap();
        let role = roles
            .iter()
            .find(|(name, _)| name == &role_name)
            .map(|(_, role)| *role)
            .ok_or("Role not found")?;

        Ok(role)
    }

    fn collect_color_candidates(&self, exclude_role: Option<Role>) -> Vec<ColorCandidate> {
        let mut candidates = Vec::new();
        let mut seen_colors: HashMap<String, Role> = HashMap::new();

        let all_roles = vec![
            Role::Heading1,
            Role::Heading2,
            Role::Heading3,
            Role::Error,
            Role::Warning,
            Role::Success,
            Role::Info,
            Role::Emphasis,
            Role::Code,
            Role::Normal,
            Role::Subtle,
            Role::Hint,
            Role::Debug,
            Role::Link,
            Role::Quote,
            Role::Commentary,
        ];

        for role in all_roles {
            let style = self.theme.style_for(role);
            if let Some(mut candidate) = ColorCandidate::from_style(&style, Some(role)) {
                // Skip the role we're editing (unless we want to keep it as an option)
                if exclude_role == Some(role) {
                    candidate.current_role = None;
                }

                // Track if we've seen this color before
                if let Some(&_first_role) = seen_colors.get(&candidate.hex) {
                    if exclude_role != Some(role) {
                        continue; // Skip duplicates
                    }
                } else {
                    seen_colors.insert(candidate.hex.clone(), role);
                }

                candidates.push(candidate);
            }
        }

        candidates
    }

    fn update_role(&mut self, role: Role, rgb: [u8; 3]) -> Result<(), Box<dyn Error>> {
        let style = self.theme.style_for(role);
        let mut new_style = Style::with_rgb(rgb);

        // Preserve attributes
        if style.bold {
            new_style = new_style.bold();
        }
        if style.italic {
            new_style = new_style.italic();
        }
        if style.dim {
            new_style = new_style.dim();
        }
        if style.underline {
            new_style = new_style.underline();
        }

        // Update the palette
        match role {
            Role::Heading1 => self.theme.palette.heading1 = new_style,
            Role::Heading2 => self.theme.palette.heading2 = new_style,
            Role::Heading3 => self.theme.palette.heading3 = new_style,
            Role::Error => self.theme.palette.error = new_style,
            Role::Warning => self.theme.palette.warning = new_style,
            Role::Success => self.theme.palette.success = new_style,
            Role::Info => self.theme.palette.info = new_style,
            Role::Emphasis => self.theme.palette.emphasis = new_style,
            Role::Code => self.theme.palette.code = new_style,
            Role::Normal => self.theme.palette.normal = new_style,
            Role::Subtle => self.theme.palette.subtle = new_style,
            Role::Hint => self.theme.palette.hint = new_style,
            Role::Debug => self.theme.palette.debug = new_style,
            Role::Link => self.theme.palette.link = new_style,
            Role::Quote => self.theme.palette.quote = new_style,
            Role::Commentary => self.theme.palette.commentary = new_style,
        }

        Ok(())
    }

    fn style_to_hex(style: &Style) -> String {
        style
            .foreground
            .as_ref()
            .and_then(|color_info| {
                if let ColorValue::TrueColor { rgb } = &color_info.value {
                    Some(format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "#000000".to_string())
    }

    fn extract_rgb(style: &Style) -> Result<[u8; 3], Box<dyn Error>> {
        style
            .foreground
            .as_ref()
            .and_then(|color_info| {
                if let ColorValue::TrueColor { rgb } = &color_info.value {
                    Some(*rgb)
                } else {
                    None
                }
            })
            .ok_or_else(|| "Could not extract RGB from style".into())
    }
}

fn load_theme(path: &Path) -> Result<Theme, Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    let theme = Theme::load_from_file(path)?;
    Ok(theme)
}

fn save_theme(theme: &Theme, path: &Path, create_backup: bool) -> Result<(), Box<dyn Error>> {
    // Create backup if requested
    if create_backup && path.exists() {
        let backup_path = path.with_extension("toml.backup");
        fs::copy(path, &backup_path)?;
        println!("📦 Backup created: {}", backup_path.display());
    }

    // Read the original file
    let content = fs::read_to_string(path)?;
    let mut doc: toml::Value = toml::from_str(&content)?;

    // Update the palette section
    if let Some(palette) = doc.get_mut("palette").and_then(|p| p.as_table_mut()) {
        update_palette_in_toml(palette, &theme.palette)?;
    }

    // Write back
    let new_content = toml::to_string_pretty(&doc)?;
    fs::write(path, new_content)?;

    println!("💾 Theme saved: {}", path.display());
    Ok(())
}

fn update_palette_in_toml(
    palette: &mut toml::map::Map<String, toml::Value>,
    new_palette: &Palette,
) -> Result<(), Box<dyn Error>> {
    let roles = vec![
        ("heading1", &new_palette.heading1),
        ("heading2", &new_palette.heading2),
        ("heading3", &new_palette.heading3),
        ("error", &new_palette.error),
        ("warning", &new_palette.warning),
        ("success", &new_palette.success),
        ("info", &new_palette.info),
        ("emphasis", &new_palette.emphasis),
        ("code", &new_palette.code),
        ("normal", &new_palette.normal),
        ("subtle", &new_palette.subtle),
        ("hint", &new_palette.hint),
        ("debug", &new_palette.debug),
        ("link", &new_palette.link),
        ("quote", &new_palette.quote),
        ("commentary", &new_palette.commentary),
    ];

    for (role_name, style) in roles {
        if let Some(color_info) = &style.foreground {
            if let ColorValue::TrueColor { rgb } = &color_info.value {
                if let Some(role_table) = palette.get_mut(role_name).and_then(|r| r.as_table_mut())
                {
                    if let Some(rgb_array) =
                        role_table.get_mut("rgb").and_then(|r| r.as_array_mut())
                    {
                        *rgb_array = vec![
                            toml::Value::Integer(i64::from(rgb[0])),
                            toml::Value::Integer(i64::from(rgb[1])),
                            toml::Value::Integer(i64::from(rgb[2])),
                        ];
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    println!("🎨 Thag Theme Editor\n");

    // Load the theme
    let theme = load_theme(&cli.input)?;

    // Create editor
    let mut editor = ThemeEditor::new(theme);

    // Run interactive editor
    editor.run()?;

    // Save if modified
    if editor.modified {
        let output_path = cli.output.as_ref().unwrap_or(&cli.input);
        save_theme(&editor.theme, output_path, cli.backup)?;
    } else {
        println!("ℹ️  No changes made.");
    }

    Ok(())
}
