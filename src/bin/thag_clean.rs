/*[toml]
[dependencies]
thag_styling = { version = "1, thag-auto", features = ["inquire_theming"] }
*/

/// `thag` prompted front-end command to clean script build artifacts.
///
/// Offers choices to show or clean script build artifacts, or sweep stale ones
/// with `cargo sweep`. The "sweep" option preserves recently-used artifacts while
/// removing stale ones, and handles multiple `thag_rs_shared_target` directories
/// (e.g. those created by AI agent sessions alongside the primary one).
//# Purpose: Interactively show, clean or sweep thag script build artifacts.
//# Categories: maintenance, thag_front_ends, tools
use chrono::{DateTime, Local};
use inquire::{set_global_render_config, validator::Validation, Confirm, CustomType, Select};
use std::{
    cmp::Reverse,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
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

/// System temporary directory path.
static TMPDIR: LazyLock<PathBuf> = LazyLock::new(env::temp_dir);

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
        "Sweep stale build artifacts           (cargo sweep)",
    ];

    loop {
        let selection = Select::new("What would you like to do?", choices.clone())
            .with_help_message("Use ↑/↓ to navigate, Enter to confirm, Esc to cancel")
            .prompt();

        let result = match selection {
            Ok(choice) if choice.starts_with("Show cached") => show_artifacts("bins"),
            Ok(choice) if choice.starts_with("Show shared") => show_artifacts("target"),
            Ok(choice) if choice.starts_with("Show all") => show_artifacts("all"),
            Ok(choice) if choice.starts_with("Show thag_url") => show_web_scripts(),
            Ok(choice) if choice.starts_with("Clean cached") => clean_artifacts("bins"),
            Ok(choice) if choice.starts_with("Clean shared") => clean_artifacts("target"),
            Ok(choice) if choice.starts_with("Clean all") => clean_artifacts("all"),
            Ok(choice) if choice.starts_with("Clean only") => clean_web_scripts(),
            Ok(_) => sweep_with_cargo_sweep(),
            Err(
                inquire::InquireError::OperationCanceled
                | inquire::InquireError::OperationInterrupted,
            ) => {
                "Cancelled.".normal().println();
                return Ok(());
            }
            Err(e) => Err(e.into()),
        };
        result?;
        println!();
    }
}

/// Find all `(project_dir, target_dir)` pairs under `$TMPDIR`.
///
/// Searches at depth 0 (direct children of `$TMPDIR`) and depth 1 (grandchildren)
/// for directories named [`PACKAGE_NAME`] that have a sibling named
/// [`SHARED_TARGET_SUBDIR`]. This catches both the normal location and any
/// AI-agent-session directories such as `zed-agent-terminal-*/thag_rs`.
fn find_project_target_pairs() -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();

    // Direct pair: $TMPDIR/thag_rs  +  $TMPDIR/thag_rs_shared_target
    let direct_proj = TMPDIR.join(PACKAGE_NAME);
    let direct_target = TMPDIR.join(SHARED_TARGET_SUBDIR);
    if direct_proj.is_dir() && direct_target.is_dir() {
        pairs.push((direct_proj, direct_target));
    }

    // One level deeper: $TMPDIR/<subdir>/thag_rs  +  $TMPDIR/<subdir>/thag_rs_shared_target
    if let Ok(entries) = fs::read_dir(&*TMPDIR) {
        for entry in entries.flatten() {
            let parent = entry.path();
            if !parent.is_dir() {
                continue;
            }
            let candidate_proj = parent.join(PACKAGE_NAME);
            let candidate_target = parent.join(SHARED_TARGET_SUBDIR);
            if candidate_proj.is_dir() && candidate_target.is_dir() {
                pairs.push((candidate_proj, candidate_target));
            }
        }
    }

    pairs
}

/// Find all `thag_rs_shared_target` directories under `$TMPDIR`, whether or not a
/// companion project directory exists alongside them.
fn find_all_target_dirs() -> Vec<PathBuf> {
    let mut found = Vec::new();

    // Direct child: $TMPDIR/thag_rs_shared_target
    let direct = TMPDIR.join(SHARED_TARGET_SUBDIR);
    if direct.is_dir() {
        found.push(direct);
    }

    // One level deeper: $TMPDIR/<subdir>/thag_rs_shared_target
    if let Ok(entries) = fs::read_dir(&*TMPDIR) {
        for entry in entries.flatten() {
            let parent = entry.path();
            if !parent.is_dir() {
                continue;
            }
            let candidate = parent.join(SHARED_TARGET_SUBDIR);
            if candidate.is_dir() {
                found.push(candidate);
            }
        }
    }

    found
}

fn show_artifacts(show_what: &str) -> Result<(), Box<dyn Error>> {
    let bins_dir = TMPDIR.join(EXECUTABLE_CACHE_SUBDIR);

    match show_what {
        "bins" => {
            if bins_dir.exists() {
                veprtln!(V::N, "Executable cache: {}", bins_dir.display());
                list_dir_and_print_top(&bins_dir, false, usize::MAX)?;
            } else {
                veprtln!(
                    V::N,
                    "Executable cache {} does not exist",
                    bins_dir.display()
                );
            }
        }
        "target" => {
            let target_dirs = find_all_target_dirs();
            if target_dirs.is_empty() {
                veprtln!(V::N, "No shared build caches found");
            } else {
                let top_n = prompt_top_n()?;
                for target_dir in &target_dirs {
                    println!();
                    veprtln!(V::N, "Shared build cache: {}", target_dir.display());
                    list_dir_and_print_top(target_dir, true, top_n)?;
                }
            }
        }
        "all" => {
            if bins_dir.exists() {
                veprtln!(V::N, "Executable cache: {}", bins_dir.display());
                list_dir_and_print_top(&bins_dir, false, usize::MAX)?;
            } else {
                veprtln!(
                    V::N,
                    "Executable cache {} does not exist",
                    bins_dir.display()
                );
            }
            let target_dirs = find_all_target_dirs();
            if target_dirs.is_empty() {
                veprtln!(V::N, "No shared build caches found");
            } else {
                let top_n = prompt_top_n()?;
                for target_dir in &target_dirs {
                    println!();
                    veprtln!(V::N, "Shared build cache: {}", target_dir.display());
                    list_dir_and_print_top(target_dir, true, top_n)?;
                }
            }
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

/// Prompt for a `top_n` value used when listing the largest artifacts.
fn prompt_top_n() -> Result<usize, Box<dyn Error>> {
    Ok(CustomType::<usize>::new(
        "How many directory entries do you want to display, ranked by size?",
    )
    .with_starting_input("20")
    .with_formatter(&|i| format!("{i}"))
    .with_error_message("Please type a valid number")
    .with_help_message("Type a positive integer")
    .with_validator(|val: &usize| {
        if *val < 1 {
            Ok(Validation::Invalid(
                "You must request at least one entry".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    })
    .prompt()?)
}

/// Accumulated file metadata used for sorting and display.
#[derive(Clone)]
struct FileInfo {
    formatted_time: String,
    file_size: u64,
    file_name: String,
}

/// Recursively collect [`FileInfo`] for every file under `dir`.
/// When `recursive` is `false`, only direct file children of `dir` are included.
fn collect_files(
    dir: &Path,
    recursive: bool,
    files: &mut Vec<FileInfo>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if metadata.is_file() {
            let file_size = metadata.len();
            let modified_time = metadata.modified()?;
            let datetime: DateTime<Local> = modified_time.into();
            let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            let file_name = path.display().to_string();
            files.push(FileInfo {
                formatted_time,
                file_size,
                file_name,
            });
        } else if metadata.is_dir() && recursive {
            collect_files(&path, true, files)?;
        }
    }
    Ok(())
}

/// Collect files from `dir_path` (recursively if requested), sort by size descending,
/// and print the top `print_top` entries.
fn list_dir_and_print_top(
    dir_path: &Path,
    recursive: bool,
    print_top: usize,
) -> Result<(), Box<dyn Error>> {
    let mut files = Vec::new();
    collect_files(dir_path, recursive, &mut files)?;
    files.sort_by_key(|f| Reverse(f.file_size));

    for file in files.iter().take(print_top) {
        println!(
            "{} {:>10} bytes  {}",
            file.formatted_time, file.file_size, file.file_name
        );
    }

    Ok(())
}

/// List all `web_script*` entries (without removing them) from the locations that
/// `thag_url` and `thag` populate when running a URL-sourced script.
fn show_web_scripts() -> Result<(), Box<dyn Error>> {
    let debug_dir = TMPDIR.join(SHARED_TARGET_SUBDIR).join("debug");
    let parent_dirs: Vec<PathBuf> = vec![
        TMPDIR.to_path_buf(),
        TMPDIR.join(PACKAGE_NAME),
        debug_dir.join(".fingerprint"),
        debug_dir.join("incremental"),
        debug_dir.join("deps"),
        debug_dir.clone(),
        TMPDIR.join(EXECUTABLE_CACHE_SUBDIR),
    ];

    let mut found_any = false;
    let mut files = Vec::new();
    for dir in &parent_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with(WEB_SCRIPT_PREFIX) {
                    // println!("  {}", entry.path().display());
                    let path = entry.path();
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let file_size = metadata.len();
                    let modified_time = metadata.modified()?;
                    let datetime: DateTime<Local> = modified_time.into();
                    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                    let file_name = path.display().to_string();
                    files.push(FileInfo {
                        formatted_time,
                        file_size,
                        file_name,
                    });
                    found_any = true;
                }
            }
        }
    }

    if !found_any {
        "No web_script* artefacts found.".normal().println();
        return Ok(());
    }

    // eprintln!("Sorting web artifacts by modification time...");
    files.sort_by_key(|f| Reverse(f.formatted_time.clone()));

    for file in files.iter()
    /* .take(print_top) */
    {
        println!(
            "{} {:>10} bytes  {}",
            file.formatted_time, file.file_size, file.file_name
        );
    }

    Ok(())
}

/// Delegates to `thag --clean <what>` for the primary build cache, then warns
/// about any additional target directories that were not covered.
fn clean_artifacts(clean_what: &str) -> Result<(), Box<dyn Error>> {
    let artifacts_desc = match clean_what {
        "bins" => "cached executables",
        "target" => "shared build artifacts",
        "all" => "all script build artifacts",
        _ => return Ok(()),
    };

    let confirmed = Confirm::new(&format!("This will delete {artifacts_desc}. Continue?"))
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
        sprtln!(Role::Success, "✓ {artifacts_desc} cleaned.");

        // Warn about any additional target dirs that `thag --clean` doesn't know about.
        if matches!(clean_what, "target" | "all") {
            let primary = TMPDIR.join(SHARED_TARGET_SUBDIR);
            let extras: Vec<_> = find_all_target_dirs()
                .into_iter()
                .filter(|d| d != &primary)
                .collect();
            if !extras.is_empty() {
                sprtln!(
                    Role::Warning,
                    "Note: the following additional target directories were not cleaned:"
                );
                for d in &extras {
                    println!("  {}", d.display());
                }
                "Use 'Sweep stale build artifacts' to clean these."
                    .warning()
                    .println();
            }
        }
    } else {
        sprtln!(Role::Error, "thag --clean exited with status: {status}");
    }

    Ok(())
}

/// Removes all files and directories whose names begin with `web_script` from the
/// locations that `thag_url` and `thag` populate when running a URL-sourced script.
fn clean_web_scripts() -> Result<(), Box<dyn Error>> {
    let debug_dir = TMPDIR.join(SHARED_TARGET_SUBDIR).join("debug");
    let parent_dirs: Vec<PathBuf> = vec![
        TMPDIR.to_path_buf(),
        TMPDIR.join(PACKAGE_NAME),
        debug_dir.join(".fingerprint"),
        debug_dir.join("incremental"),
        debug_dir.join("deps"),
        debug_dir.clone(),
        TMPDIR.join(EXECUTABLE_CACHE_SUBDIR),
    ];

    let mut to_remove: Vec<PathBuf> = Vec::new();
    for dir in &parent_dirs {
        if !dir.exists() {
            continue;
        }
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with(WEB_SCRIPT_PREFIX) {
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

/// Sweeps stale build artifacts using `cargo sweep`.
///
/// Finds all `(project_dir, target_dir)` pairs under `$TMPDIR` — including any
/// created by AI agent sessions — prompts for a staleness threshold in days, then
/// runs `cargo sweep --recursive --time <days>` on each project directory with
/// `CARGO_TARGET_DIR` set to its companion shared target directory so that
/// `cargo sweep` uses the correct (shared) target rather than a per-project one.
fn sweep_with_cargo_sweep() -> Result<(), Box<dyn Error>> {
    // Verify cargo-sweep is available before prompting for parameters.
    match Command::new("cargo").args(["sweep", "--version"]).output() {
        Ok(out) if out.status.success() => {}
        _ => {
            sprtln!(Role::Error, "cargo-sweep is not installed or not working.");
            "Install it with: cargo install cargo-sweep"
                .normal()
                .println();
            return Ok(());
        }
    }

    let pairs = find_project_target_pairs();

    if pairs.is_empty() {
        "No thag build directories found.".normal().println();
        return Ok(());
    }

    sprtln!(Role::Heading2, "\nFound {} build location(s):", pairs.len());
    for (proj, target) in &pairs {
        println!("  project dir: {}", proj.display());
        println!("  target dir:  {}", target.display());
    }
    println!();

    let days = CustomType::<u32>::new("Remove artifacts not accessed within how many days?")
        .with_starting_input("14")
        .with_formatter(&|i| format!("{i}"))
        .with_error_message("Please type a valid number of days")
        .with_help_message("Type a positive integer (e.g. 14 for two weeks)")
        .with_validator(|val: &u32| {
            if *val == 0 {
                Ok(Validation::Invalid("Must be at least 1 day".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;

    let confirmed = Confirm::new(&format!(
        "Sweep artifacts not accessed in the last {days} day(s) from {} location(s)?",
        pairs.len()
    ))
    .with_default(false)
    .prompt()?;

    if !confirmed {
        "Cancelled.".normal().println();
        return Ok(());
    }

    for (proj_dir, target_dir) in &pairs {
        sprtln!(Role::INFO, "Sweeping {} ...", proj_dir.display());
        let status = Command::new("cargo")
            .args(["sweep", "--recursive", "--time", &days.to_string()])
            .arg(proj_dir)
            .env("CARGO_TARGET_DIR", target_dir)
            .status()?;

        if status.success() {
            sprtln!(Role::Success, "✓ Swept {}.", proj_dir.display());
        } else {
            sprtln!(
                Role::Error,
                "cargo sweep failed for {}.",
                proj_dir.display()
            );
        }
    }

    Ok(())
}
