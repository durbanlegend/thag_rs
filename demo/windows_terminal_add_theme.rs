use serde_json::Value;
use std::{fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    // Path to your Windows Terminal settings.json
    let mut settings_path = dirs::data_local_dir().expect("No local data dir found");
    settings_path
        .push(r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json");

    // Path to your themes directory
    let themes_dir = PathBuf::from("themes");

    // Load settings.json
    let mut settings: Value = serde_json::from_str(&fs::read_to_string(&settings_path)?)?;

    // Load all theme files
    let mut new_schemes = Vec::new();
    for entry in fs::read_dir(&themes_dir)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .map(|e| e == "json")
            .unwrap_or(false)
        {
            let theme: Value = serde_json::from_str(&fs::read_to_string(entry.path())?)?;
            new_schemes.push(theme);
        }
    }

    // Ensure "schemes" exists and is an array
    let schemes = settings
        .as_object_mut()
        .unwrap()
        .entry("schemes")
        .or_insert_with(|| Value::Array(Vec::new()));

    let array = schemes.as_array_mut().unwrap();

    // Add new schemes if they don't already exist (by name)
    for scheme in new_schemes {
        if let Some(name) = scheme.get("name").and_then(|n| n.as_str()) {
            let exists = array
                .iter()
                .any(|s| s.get("name").and_then(|n| n.as_str()) == Some(name));
            if !exists {
                array.push(scheme);
            }
        }
    }

    // Backup old settings.json
    let backup_path = settings_path.with_extension("bak");
    fs::copy(&settings_path, &backup_path)?;

    // Write updated settings.json
    fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    println!(
        "✅ Added new schemes, now {} total custom schemes",
        array.len()
    );
    Ok(())
}
