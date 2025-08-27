/*[toml]
[dependencies]
thag_proc_macros = { version = "0.2, thag-auto" }
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["tools"] }
*/

use inquire::{self, set_global_render_config};
use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;
use thag_proc_macros::file_navigator;
use thag_rs::{auto_help, help_system::check_help_and_exit, themed_inquire_config};
/// Demo directory downloader. Very fast replacement for `thag_get_demo` with subdirectory
/// support so as to include the `demo/proc_macros` directory. Git `sparse-checkout`
/// approach suggested and written by `ChatGPT`, local directory handling assisted by Claude.
//# Purpose: Prototype for `thag_get_demo_dir`.
//# Categories: crates, prototype, technique
use thag_styling::Styler;

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
    // Check for help first - automatically extracts from source comments
    let help = auto_help!("thag_get_demo_dir");
    check_help_and_exit(&help);

    set_global_render_config(themed_inquire_config());

    // 1) Select a target parent directory
    let mut navigator = FileNavigator::new();

    let (dest_dir, demo_dest) = loop {
        println!("Select where you want to save the new `demo` directory");
        let Ok(dest_dir) = select_directory(&mut navigator, false) else {
            println!("\nNo directory selected. Exiting.\n");
            return Ok(());
        };
        // // `select_directory` already handles this.
        // fs::create_dir_all(&dest_dir)?;

        // Check if demo already exists in destination
        let demo_dest = dest_dir.join("demo");
        if demo_dest.exists() {
            println!(
                "\n{}\n",
                format!("Destination already has subdirectory {}", "demo")
                    .style()
                    .emphasis()
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
            "main",
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
