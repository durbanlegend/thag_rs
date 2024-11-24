use std::error::Error;
use std::fmt;
use std::process::{Command, Stdio};

#[derive(Debug)]
struct UrlError(String);

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for UrlError {}

enum SourceType {
    GitHub,
    GitLab,
    Bitbucket,
    RustPlayground,
    Raw,
}

fn detect_source_type(url: &str) -> SourceType {
    if url.contains("github.com") {
        SourceType::GitHub
    } else if url.contains("gitlab.com") {
        SourceType::GitLab
    } else if url.contains("bitbucket.org") {
        SourceType::Bitbucket
    } else if url.contains("play.rust-lang.org") {
        SourceType::RustPlayground
    } else {
        SourceType::Raw
    }
}

fn convert_to_raw_url(url: &str) -> Result<String, UrlError> {
    match detect_source_type(url) {
        SourceType::GitHub => {
            if url.contains("/blob/") {
                Ok(url
                    .replace("github.com", "raw.githubusercontent.com")
                    .replace("/blob/", "/"))
            } else {
                Err(UrlError("Invalid GitHub URL format".to_string()))
            }
        }
        SourceType::GitLab => {
            if url.contains("/-/blob/") {
                Ok(url.replace("/-/blob/", "/-/raw/"))
            } else {
                Err(UrlError("Invalid GitLab URL format".to_string()))
            }
        }
        SourceType::Bitbucket => {
            if url.contains("/src/") {
                Ok(url.replace("/src/", "/raw/"))
            } else {
                Err(UrlError("Invalid Bitbucket URL format".to_string()))
            }
        }
        SourceType::RustPlayground => {
            // Extract gist ID from Playground URL
            if let Some(gist_id) = url.split("gist=").nth(1) {
                Ok(format!(
                    "https://gist.githubusercontent.com/rust-play/{}/raw",
                    gist_id
                ))
            } else {
                Err(UrlError(
                    "Invalid Rust Playground URL format. Expected URL with gist ID".to_string(),
                ))
            }
        }
        SourceType::Raw => Ok(url.to_string()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} [-s|-d] <url>", args[0]);
        eprintln!("Supported sources:");
        eprintln!("  - GitHub (github.com)");
        eprintln!("  - GitLab (gitlab.com)");
        eprintln!("  - Bitbucket (bitbucket.org)");
        eprintln!("  - Rust Playground (play.rust-lang.org)");
        eprintln!("  - Raw URLs (direct links to raw content)");
        std::process::exit(1);
    }

    let (flag, url) = if args.len() == 3 {
        let flag = match args[1].as_str() {
            "-s" | "-d" => args[1].as_str(),
            _ => {
                eprintln!("Invalid flag. Use -s or -d");
                std::process::exit(1);
            }
        };
        (flag, &args[2])
    } else {
        ("-s", &args[1]) // default to -s if no flag provided
    };

    let raw_url = convert_to_raw_url(url)?;

    // Create the curl command
    let curl = Command::new("curl")
        .args(["-sL", &raw_url])
        .stdout(Stdio::piped())
        .spawn()?;

    // Pipe curl's output to thag
    let status = Command::new("thag")
        .arg(flag)
        .stdin(curl.stdout.unwrap())
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_github_url() {
        let url = "https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs";
        let expected =
            "https://raw.githubusercontent.com/durbanlegend/thag_rs/master/demo/hello.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_gitlab_url() {
        let url = "https://gitlab.com/user/repo/-/blob/master/file.rs";
        let expected = "https://gitlab.com/user/repo/-/raw/master/file.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_bitbucket_url() {
        let url = "https://bitbucket.org/user/repo/src/master/file.rs";
        let expected = "https://bitbucket.org/user/repo/raw/master/file.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_playground_url() {
        let url = "https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=362dc87d7c1c8f2d569cc205165424d3";
        let expected =
            "https://gist.githubusercontent.com/rust-play/362dc87d7c1c8f2d569cc205165424d3/raw";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_raw_url() {
        let url = "https://example.com/raw/file.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), url);
    }
}
