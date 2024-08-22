use reqwest::blocking::get;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;

use crate::logging::Verbosity;
use crate::{log, nu_color_println, nu_resolve_style, BuildRunError};

#[derive(Deserialize)]
struct GitHubFile {
    name: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
}

fn get_github_files(repo: &str, path: &str) -> Result<Vec<GitHubFile>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "request")
        .send()?
        .json::<Vec<GitHubFile>>()?;

    Ok(response)
}

fn download_demo_files(repo: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dest_dir = PathBuf::from(path);
    if !dest_dir.exists() {
        nu_color_println!(
            nu_resolve_style(crate::MessageLevel::Warning),
            "No such directory"
        );
        return Err(Box::new(BuildRunError::Command(
            "No such directory".to_string(),
        )));
    }
    let mut chosen = false;
    while !chosen {
        // Display file chooser dialogue to select the destination directory
        let dest_dir = FileDialog::new()
            .set_title("Choose a destination directory for demo files")
            .set_directory(".")
            .pick_folder()
            .ok_or("No folder selected")?;
        let is_empty = dest_dir.read_dir()?.next().is_none();
        let dest_dir_str = dest_dir.display();
        if !is_empty {
            println!(
                r"{}{} already contains files. Any matching files will be overwritten.
Are you sure you want to proceed? Y/n",
                nu_ansi_term::Color::Magenta.bold().paint("Warning:"),
                dest_dir_str
            );
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            chosen = input == "y" || input == "Y";
        }
        if chosen {
            println!("Downloading starter kit files to {dest_dir_str} ...");
            let files = get_github_files(repo, path)?;

            for file in files {
                if file.file_type == "file" {
                    if let Some(download_url) = file.download_url {
                        let response = get(&download_url)?;
                        let dest_path = dest_dir.join(&file.name);
                        let mut dest_file = File::create(dest_path)?;
                        println!(
                            "{} {}",
                            nu_ansi_term::Color::Green.bold().paint("Downloading"),
                            file.name
                        );
                        copy(&mut response.bytes()?.as_ref(), &mut dest_file)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let repo = "durbanlegend/thag_rs";
    let path = "demo";

    match download_demo_files(repo, path) {
        Ok(_) => println!("Demo files downloaded successfully."),
        Err(e) => eprintln!("Error downloading demo files: {}", e),
    }
}