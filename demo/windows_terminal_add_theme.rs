use serde_json::Value;
use std::{env, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    // Path to your Windows Terminal settings.json
    let mut settings_path = dirs::data_local_dir()
        .expect("No local data dir found");
    settings_path.push(
        r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"
    );

    dbg!(&settings_path);
    dbg!(&settings_path.exists());
    // Load settings.json
    let mut settings: Value = serde_json::from_str(&fs::read_to_string(&settings_path)?)?;
    // dbg!(&settings);

    // Collect new schemes
    let args: Vec<String> = env::args().skip(1).collect();
    dbg!(&args);

    let mut new_schemes = Vec::new();

    let themes_dir = PathBuf::from("exported_themes");
    if args.is_empty() {
        // Load all theme files from "themes" dir
        for entry in fs::read_dir(&themes_dir)? {
            let entry = entry?;
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                let theme: Value = serde_json::from_str(&fs::read_to_string(entry.path())?)?;
                eprintln!("theme={theme:?}");
                new_schemes.push(theme);
            }
        }
    } else {
        // Each argument is a JSON file path
        dbg!(&themes_dir);
        for arg in args {
            let theme_path = themes_dir.join(arg);
            let theme: Value = serde_json::from_str(&fs::read_to_string(&theme_path)?)?;
            new_schemes.push(theme);
        }
    }

    let total_schemes;
    {
        // Scoped mutable borrow
        let schemes = settings
            .as_object_mut()
            .unwrap()
            .entry("schemes")
            .or_insert_with(|| Value::Array(Vec::new()));

        let array = schemes.as_array_mut().unwrap();

        for scheme in new_schemes {
            if let Some(name) = scheme.get("name").and_then(|n| n.as_str()) {
                let exists = array.iter().any(|s| {
                    s.get("name").and_then(|n| n.as_str()) == Some(name)
                });
                if !exists {
                    array.push(scheme);
                }
            }
        }

        total_schemes = array.len();
    } // <-- mutable borrow ends here

    // Backup old settings.json
    let backup_path = settings_path.with_extension("bak");
    fs::copy(&settings_path, &backup_path)?;

    // Write updated settings.json
    fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    println!("✅ Added new schemes, now {} total custom schemes", total_schemes);
    Ok(())
}
