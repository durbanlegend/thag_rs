/*[toml]
[dependencies]
# The `thag` command uses the `thag-auto` keyword here to resolve dependencies automatically based on your environment:
# - Default: Uses crates.io (no environment variables needed)
# - Development: Set THAG_DEV_PATH=/absolute/path/to/thag_rs (e.g. $PWD not .)
# - Git: Set THAG_GIT_REF=main (or other branch) to use git repository instead of crates.io
# E.g. from `thag_rs` project dir: `THAG_DEV_PATH=$PWD thag demo/download_demos.rs`
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
*/

/// Prototype script for `thag_get_demo_dir` - fast replacement for `thag_get_demo`
/// with subdirectory support. Git `sparse-checkout` approach suggested and written
/// by ChatGPT, local directory handling assisted by Claude.
//# Purpose: Prototype for `thag_get_demo_dir`.
//# Categories: crates, prototype, technique
use colored::Colorize;
use inquire;
use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;
use thag_proc_macros::file_navigator;

file_navigator! {}

struct TempCleanupGuard(PathBuf);

impl Drop for TempCleanupGuard {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1) Select a target parent directory
    let mut navigator = FileNavigator::new();

    let (dest_dir, demo_dest) = loop {
        println!("Select where you want to save the new `demo` directory");
        let dest_dir = match select_directory(&mut navigator, false) {
            Ok(path) => path,
            Err(_) => {
                println!("\nNo directory selected. Exiting.\n");
                return Ok(());
            }
        };

        // // `select_directory` already handles this.
        // fs::create_dir_all(&dest_dir)?;

        // Check if demo already exists in destination
        let demo_dest = dest_dir.join("demo");
        if demo_dest.exists() {
            println!(
                "\n{}\n",
                format!("Destination already has subdirectory {}", "demo".bold())
                    // .bold()
                    .magenta()
            );
            continue;
        }
        break (dest_dir, demo_dest);
    };

    // We'll do a temporary clone as a sibling of the destination demo directory
    let temp_clone = dest_dir.join("temp_thag_rs_clone");
    let _cleanup_guard = TempCleanupGuard(temp_clone.clone());

    // 2) Sparse clone into temporary dir
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            "develop",
            "--filter=blob:none",
            "--sparse",
            "https://github.com/durbanlegend/thag_rs.git",
            temp_clone.to_str().unwrap(),
        ])
        .status()?;
    if !status.success() {
        return Err("git clone failed".into());
    }

    // 3) Set sparse-checkout to include demo/
    let status = Command::new("git")
        .current_dir(&temp_clone)
        .args(["sparse-checkout", "set", "--no-cone", "demo"])
        .status()?;
    if !status.success() {
        return Err("git sparse-checkout failed".into());
    }

    // 4) Exclude demo/proc_macros/target/
    let sparse_file = temp_clone.join(".git/info/sparse-checkout");
    let mut file = fs::OpenOptions::new().append(true).open(&sparse_file)?;
    file.write_all(b"!demo/proc_macros/target/\n")?;

    // 5) Apply updated sparse patterns
    let status = Command::new("git")
        .current_dir(&temp_clone)
        .args(["read-tree", "-mu", "HEAD"])
        .status()?;
    if !status.success() {
        return Err("git read-tree failed".into());
    }

    // 6) Remove .git to leave plain files
    let status = Command::new("rm")
        .args(["-rf", ".git"])
        .current_dir(&temp_clone)
        .status()?;
    if !status.success() {
        return Err("failed to remove .git".into());
    }

    // 7) Move demo/ directory to the user-specified destination
    let demo_src = temp_clone.join("demo");
    fs::rename(&demo_src, &demo_dest)?;

    // 8) Remove the temporary clone directory
    fs::remove_dir_all(&temp_clone)?;

    println!("✅ demo/ downloaded to: {}", demo_dest.display());
    Ok(())
}
