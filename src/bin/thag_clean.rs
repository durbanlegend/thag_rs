/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
*/

/// `thag` prompted front-end command to clean script build artifacts.
///
/// Offers a choice between cleaning all script build artifacts (via `thag --clean`)
/// or cleaning only the build artifacts left by `thag_url`, which are identified by
/// the `web_script*` prefix in the system temporary directory.
//# Purpose: Interactively clean thag script build artifacts, with option to target only thag_url leftovers.
//# Categories: maintenance, thag_front_ends, tools
use chrono::{DateTime, Local};
use inquire::{set_global_render_config, Confirm, Select};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    result::Result,
    sync::LazyLock,
};
use thag_styling::{
    auto_help, help_system::check_help_and_exit, sprtln, themed_inquire_config, veprtln, Role,
    Styleable, StyledPrint, V,
};

/// Prefix used by `thag_url` when creating temporary script files.
const WEB_SCRIPT_PREFIX: &str = "web_script";

/// Subdirectory name for the per-script Cargo projects inside `$TMPDIR`.
const PACKAGE_NAME: &str = "thag_rs";

/// Subdirectory name for the shared Cargo target directory.
const SHARED_TARGET_SUBDIR: &str = "thag_rs_shared_target";

/// Subdirectory name for the cached script executables.
const EXECUTABLE_CACHE_SUBDIR: &str = "thag_rs_bins";

/// System temporary directory path
pub static TMPDIR: LazyLock<PathBuf> = LazyLock::new(env::temp_dir);
fn main() -> Result<(), Box<dyn Error>> {
    let help = auto_help!();
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    sprtln!(
        Role::Heading1,
        "\n🧹 thag_clean — Script Build Artifact Cleaner"
    );

    let choices = vec![
        "Show cached script executables",
        "Show shared script build cache",
        "Show all script build artifacts",
        "Show thag_url artifacts",
        "Clean cached script executables only  (thag --clean bins)",
        "Clean shared script build cache       (thag --clean target)",
        "Clean all script build artifacts      (thag --clean all)",
        "Clean only thag_url artifacts         (web_script* files/dirs)",
    ];

    loop {
        let selection = Select::new("What would you like to do?", choices.clone())
            .with_help_message("Use ↑/↓ to navigate, Enter to confirm, Esc to cancel")
            .prompt();

        let result = match selection {
            Ok(choice) if choice.starts_with("Show cached") => show_artifacts("bins"),
            Ok(choice) if choice.starts_with("Show shared") => show_artifacts("target"),
            Ok(choice) if choice.starts_with("Show all") => show_artifacts("all"),
            Ok(choice) if choice.starts_with("Clean cached") => clean_artifacts("bins"),
            Ok(choice) if choice.starts_with("Clean shared") => clean_artifacts("target"),
            Ok(choice) if choice.starts_with("Clean all") => clean_artifacts("all"),
            Ok(_) => clean_web_scripts(),
            Err(
                inquire::InquireError::OperationCanceled
                | inquire::InquireError::OperationInterrupted,
            ) => {
                "Cancelled.".normal().println();
                return Ok(());
            }
            Err(e) => Err(e.into()),
        };
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        println!()
    }
}

fn show_artifacts(show_what: &str) -> Result<(), Box<dyn Error>> {
    // use std::process::Command;

    // let artifacts_desc = match show_what {
    //     "bins" => "cached executables",
    //     "target" => "shared build artifacts",
    //     "all" => "script build artifacts",
    //     _ => return Ok(()),
    // };

    let bins_dir = TMPDIR.join(EXECUTABLE_CACHE_SUBDIR);
    let target_dir = TMPDIR.join(SHARED_TARGET_SUBDIR);

    match show_what {
        "bins" => {
            if bins_dir.exists() {
                veprtln!(V::N, "Showing executable cache: {}", bins_dir.display());
                list_dir(&bins_dir)?;
            } else {
                veprtln!(V::N, "Executable cache does not exist");
            }
        }
        "target" => {
            if target_dir.exists() {
                veprtln!(V::N, "Showing shared build cache: {}", target_dir.display());
                list_dir(&target_dir)?;
            } else {
                veprtln!(V::N, "Shared build cache does not exist");
            }
        }
        "all" => {
            if bins_dir.exists() {
                veprtln!(V::N, "Listing executable cache: {}", bins_dir.display());
                list_dir(&bins_dir)?;
            } else {
                veprtln!(V::N, "Executable cache does not exist");
            }
            println!();
            if target_dir.exists() {
                veprtln!(V::N, "Listing shared build cache: {}", target_dir.display());
                list_dir(&target_dir)?;
            } else {
                veprtln!(V::N, "Shared build cache does not exist");
            };
        }
        _ => {
            return Err(format!(
                "Invalid list option: '{show_what}'. Use 'bins', 'target', or 'all'"
            )
            .into());
        }
    }

    Ok(())
}

// Define a simple struct to hold the file details
struct FileInfo {
    formatted_time: String,
    file_size: u64,
    file_name: String,
}

fn list_dir(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    // Read the entries in the directory
    let mut files = Vec::new();

    // Read the entries in the directory and collect relevant metadata
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = entry.metadata()?;
            let file_size = metadata.len();

            let modified_time = metadata.modified()?;
            let datetime: DateTime<Local> = modified_time.into();
            let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            files.push(FileInfo {
                formatted_time,
                file_size,
                file_name,
            });
        }
    }

    // Sort descending by file size (largest to smallest)
    files.sort_by(|a, b| b.file_size.cmp(&a.file_size));

    // Print the sorted output
    for file in files {
        println!(
            "[{}] {:>10} bytes  {}",
            file.formatted_time, file.file_size, file.file_name
        );
    }

    Ok(())
}

// Delegates to `thag --clean all`.
fn clean_artifacts(clean_what: &str) -> Result<(), Box<dyn Error>> {
    use std::process::Command;

    let artifacts_desc = match clean_what {
        "bins" => "cached executables",
        "target" => "shared build artifacts",
        "all" => "script build artifacts",
        _ => return Ok(()),
    };
    let confirmed = Confirm::new(&format!("This will delete all {artifacts_desc}. Continue?"))
        .with_default(false)
        .prompt()?;

    if !confirmed {
        "Cancelled.".normal().println();
        return Ok(());
    }

    sprtln!(Role::INFO, "Running: thag --clean {clean_what}");
    let status = Command::new("thag")
        .args(["--clean", clean_what])
        .status()?;

    if status.success() {
        "✓ All {artifacts_desc} cleaned.".success().println();
    } else {
        sprtln!(Role::Error, "thag --clean exited with status: {status}");
    }

    Ok(())
}

/// Removes all files and directories whose names begin with `web_script` from the
/// locations that `thag_url` and `thag` populate when running a URL-sourced script.
fn clean_web_scripts() -> Result<(), Box<dyn Error>> {
    let tmpdir = std::env::temp_dir();

    // All parent directories that may contain web_script* entries.
    let debug_dir = tmpdir.join(SHARED_TARGET_SUBDIR).join("debug");

    let parent_dirs: Vec<PathBuf> = vec![
        // Temporary .rs source file created by thag_url
        tmpdir.clone(),
        // Per-script Cargo project dir (Cargo.toml, Cargo.lock, optional generated .rs)
        tmpdir.join(PACKAGE_NAME),
        // Cargo build artefacts
        debug_dir.join(".fingerprint"),
        debug_dir.join("incremental"),
        debug_dir.join("deps"),
        // Executables and .d files produced by the build
        debug_dir.clone(),
        // Cached executables used by thag's fast-launch path
        tmpdir.join(EXECUTABLE_CACHE_SUBDIR),
    ];

    // Collect what we would remove before touching anything.
    let mut to_remove: Vec<PathBuf> = Vec::new();
    for dir in &parent_dirs {
        if !dir.exists() {
            continue;
        }
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(WEB_SCRIPT_PREFIX) {
                        to_remove.push(entry.path());
                    }
                }
            }
            Err(e) => {
                sprtln!(Role::Warning, "Could not read {}: {e}", dir.display());
            }
        }
    }

    if to_remove.is_empty() {
        "No web_script* artefacts found — nothing to do."
            .normal()
            .println();
        return Ok(());
    }

    sprtln!(
        Role::Heading2,
        "\nFound {} item(s) to remove:",
        to_remove.len()
    );
    for path in &to_remove {
        println!("  {}", path.display());
    }

    let confirmed = Confirm::new("Remove all of the above?")
        .with_default(false)
        .prompt()?;

    if !confirmed {
        "Cancelled.".normal().println();
        return Ok(());
    }

    let mut removed = 0usize;
    let mut errors = 0usize;
    for path in &to_remove {
        let result = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        match result {
            Ok(()) => {
                removed += 1;
                sprtln!(Role::INFO, "Removed: {}", path.display());
            }
            Err(e) => {
                errors += 1;
                sprtln!(Role::Error, "Failed to remove {}: {e}", path.display());
            }
        }
    }

    println!();
    if errors == 0 {
        format!("✓ Removed {removed} item(s) successfully.")
            .success()
            .println();
    } else {
        format!("Removed {removed} item(s); {errors} error(s) — see above.")
            .warning()
            .println();
    }

    Ok(())
}
