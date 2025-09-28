/*[toml]
[dependencies]
thag_rs = { version = "0.2, thag-auto", default-features = false, features = ["core", "simplelog"] }
*/
/// `thag` front-end command to run scripts from URLs.
///
/// Usage:
///
/// ```bash
/// thag_url <url> [additional_thag_args] [-- <script_args>]
/// ```
///
/// Example:
///
/// ```bash
/// thag_url https://github.com/clap-rs/clap/blob/master/examples/demo.rs -- --name "is this the Krusty Krab?"
/// ```
///
//# Purpose: A front-end to allow `thag` to run scripts from URLs while offloading network dependencies from `thag` itself.
//# Categories: technique, thag_front_ends, tools
use std::{error::Error, process::Command, string::ToString};
use syn::{parse_file, Expr};
use thag_rs::{auto_help, help_system::check_help_and_exit};
use url::Url;

enum SourceType {
    GitHub,
    GitLab,
    Bitbucket,
    RustPlayground,
    Raw,
}

#[derive(Debug)]
enum UrlError {
    Http(String),
    ParseError(String),
    SyntaxError(String),
}

impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP Error: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse Error: {msg}"),
            Self::SyntaxError(msg) => write!(f, "Syntax Error: {msg}"),
        }
    }
}

impl Error for UrlError {}

fn validate_rust_content(content: &str) -> Result<(), UrlError> {
    // Try parsing as a complete file first
    if parse_file(content).is_ok() {
        return Ok(());
    }

    // Try parsing as an expression
    // if syn::parse_str::<syn::Expr>(content).is_ok() {
    //     return Ok(());
    // }
    if extract_ast_expr(content).is_ok() {
        return Ok(());
    }

    // eprintln!("content={content}");
    // If parsing failed, format the content for better error display
    let temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| UrlError::ParseError(format!("Failed to create temp file: {e}")))?;

    std::fs::write(temp_file.path(), content)
        .map_err(|e| UrlError::ParseError(format!("Failed to write content: {e}")))?;

    // Run rustfmt on the content
    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(temp_file.path())
        .output()
        .map_err(|e| UrlError::ParseError(format!("Failed to run rustfmt: {e}")))?;

    // Display formatted content
    println!("Invalid Rust syntax. Formatted content:");
    println!("{}", "─".repeat(40));
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", "─".repeat(40));

    Err(UrlError::SyntaxError(
        "Content is not valid Rust code".to_string(),
    ))
}

/// Extracts the syn AST expression for a Rust code section.
///
/// # Errors
///
/// This function will bubble up any `syn` parse errors encountered.
pub fn extract_ast_expr(rs_source: &str) -> Result<Expr, syn::Error> {
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // vprtln!(V::N, "str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

fn fetch_and_validate(url: &str) -> Result<String, UrlError> {
    let response = tinyget::get(url)
        .send()
        .map_err(|e| UrlError::Http(format!("Failed to fetch URL: {e}")))?;

    // eprintln!(
    //     "response={response:#?}, response.status_code={}",
    //     response.status_code
    // );

    if response.status_code != 200 {
        return Err(UrlError::Http(format!(
            "HTTP {} - {}",
            response.status_code, response.reason_phrase
        )));
    }

    let content = response
        .as_str()
        .map_err(|e| UrlError::Http(format!("Failed to read response: {e}")))?;

    // Validate content before returning
    validate_rust_content(content)?;

    Ok(content.to_string())
}

fn detect_source_type(url: &Url) -> SourceType {
    url.host_str().map_or(SourceType::Raw, |host| match host {
        "github.com" => SourceType::GitHub,
        "gitlab.com" => SourceType::GitLab,
        "bitbucket.org" => SourceType::Bitbucket,
        "play.rust-lang.org" => SourceType::RustPlayground,
        _ => SourceType::Raw,
    })
}

fn convert_to_raw_url(url_str: &str) -> Result<String, UrlError> {
    let url = Url::parse(url_str).map_err(|e| UrlError::ParseError(format!("Invalid URL: {e}")))?;

    match detect_source_type(&url) {
        SourceType::GitHub => {
            let path = url.path();

            if path.contains("/raw/") {
                return Ok(url_str.to_string());
            }

            if !path.contains("/blob/") {
                return Err(UrlError::SyntaxError(
                    "GitHub URL must contain '/blob/' in path".to_string(),
                ));
            }
            if path.split('/').count() < 4 {
                return Err(UrlError::SyntaxError(
                    "Invalid GitHub URL format: expected user/repo/blob/path".to_string(),
                ));
            }
            let raw_url = url_str
                .replace("github.com", "raw.githubusercontent.com")
                .replace("/blob/", "/");
            eprintln!("raw_url={raw_url}");
            Ok(raw_url)
        }
        SourceType::GitLab => {
            let path = url.path();
            if !path.contains("/-/blob/") {
                return Err(UrlError::SyntaxError(
                    "GitLab URL must contain '/-/blob/' in path".to_string(),
                ));
            }
            if path.split('/').count() < 5 {
                return Err(UrlError::SyntaxError(
                    "Invalid GitLab URL format: expected user/repo/-/blob/path".to_string(),
                ));
            }
            Ok(url_str.replace("/-/blob/", "/-/raw/"))
        }
        SourceType::Bitbucket => {
            let path = url.path();
            if !path.contains("/src/") {
                return Err(UrlError::SyntaxError(
                    "Bitbucket URL must contain '/src/' in path".to_string(),
                ));
            }
            if path.split('/').count() < 4 {
                return Err(UrlError::SyntaxError(
                    "Invalid Bitbucket URL format: expected user/repo/src/path".to_string(),
                ));
            }
            Ok(url_str.replace("/src/", "/raw/"))
        }
        SourceType::RustPlayground => {
            let gist_id = url
                .query_pairs()
                .find(|(key, _)| key == "gist")
                .map(|(_, value)| value.to_string())
                .ok_or_else(|| {
                    UrlError::SyntaxError("No gist ID found in Playground URL".to_string())
                })?;

            if gist_id.len() != 32 {
                // Standard GitHub gist ID length
                return Err(UrlError::SyntaxError("Invalid gist ID format".to_string()));
            }

            Ok(format!(
                "https://gist.githubusercontent.com/rust-play/{gist_id}/raw"
            ))
        }
        SourceType::Raw => Ok(url_str.to_string()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    let args: Vec<String> = std::env::args().collect();

    // Need at least URL and optionally additional flags
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    eprintln!("args={args:#?}");

    // Parse arguments
    let mut iter = args.iter().skip(1); // skip program name
    let mut url = String::new();
    let mut additional_args = Vec::new();
    let mut found_separator = false;

    for arg in iter.by_ref() {
        match arg.as_str() {
            "--" => {
                found_separator = true;
                additional_args.push("--".to_string());
                break;
            }
            arg => {
                if url.is_empty() {
                    url = arg.to_string();
                } else {
                    additional_args.push(arg.to_string());
                }
            }
        }
    }

    // Collect remaining args after --
    if found_separator {
        additional_args.extend(iter.map(ToString::to_string));
    }

    if url.is_empty() {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let raw_url = convert_to_raw_url(&url)?;

    match fetch_and_validate(&raw_url) {
        Ok(content) => {
            // Create a temporary file to save the script
            let temp_dir = std::env::temp_dir();
            let pid = std::process::id();
            let temp_file_path = temp_dir.join(format!("web_script_{pid}.rs"));
            let temp_ws_dir = temp_dir.join(format!("thag_rs/web_script_{pid}"));

            // Write content to temporary file
            std::fs::write(&temp_file_path, &content)?;

            eprintln!("Created temporary script at: {}", temp_file_path.display());
            eprintln!("additional_args={additional_args:#?}");

            // Run thag with the temporary file instead of using stdin
            let mut child = Command::new("thag")
                .arg(&temp_file_path)
                .args(&additional_args)
                .spawn()?;

            // Wait for thag to complete
            let status = child.wait()?;

            // Clean up the temporary file
            eprintln!("Removing temporary script at: {}", temp_file_path.display());
            if let Err(e) = std::fs::remove_file(&temp_file_path) {
                eprintln!("Warning: Could not remove temporary directory: {e}");
            }

            // Clean up the temporary target directory
            eprintln!(
                "Removing temporary target directory at: {}",
                temp_ws_dir.display()
            );
            if let Err(e) = std::fs::remove_dir_all(&temp_ws_dir) {
                panic!("Warning: Could not remove temporary target directory: {e}");
            }

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
            Ok(())
        }
        Err(e) => {
            match e {
                UrlError::Http(ref msg) => {
                    eprintln!("Failed to fetch content: {msg}");
                }
                UrlError::ParseError(ref msg) => {
                    eprintln!("Failed to process content: {msg}");
                }
                UrlError::SyntaxError(ref msg) => {
                    eprintln!("Invalid Rust code: {msg}");
                }
            }
            std::process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {program} <url> [additional_thag_args] [-- <script_args>]");
    eprintln!("Supported sources:");
    eprintln!("  - GitHub (github.com)");
    eprintln!("  - GitLab (gitlab.com)");
    eprintln!("  - Bitbucket (bitbucket.org)");
    eprintln!("  - Rust Playground (play.rust-lang.org)");
    eprintln!("  - Raw URLs (direct links to raw content)");
    eprintln!("\nExamples:");
    eprintln!("  {program} https://github.com/user/repo/blob/master/script.rs -- -m");
    eprintln!("  {program} https://github.com/user/repo/blob/master/script.rs -v");
    eprintln!(
        "  {program} https://github.com/user/repo/blob/master/script.rs -- script_arg1 script_arg2"
    );
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_github_blob_url() {
        let url = "https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs";
        let expected =
            "https://raw.githubusercontent.com/durbanlegend/thag_rs/master/demo/hello.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_github_raw_url() {
        let raw_url = "https://github.com/mikaelmello/inquire/raw/refs/heads/main/inquire/examples/complex_autocompletion.rs";
        assert_eq!(convert_to_raw_url(raw_url).unwrap().as_str(), raw_url);
    }

    #[test]
    fn test_gitlab_url() {
        // Example from gitlab.com/rust-embedded/cortex-m
        let url = "https://gitlab.com/rust-embedded/cortex-m/-/blob/master/src/lib.rs";
        let expected = "https://gitlab.com/rust-embedded/cortex-m/-/raw/master/src/lib.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), expected);
    }

    #[test]
    fn test_bitbucket_url() {
        // Example from bitbucket.org/atlassian/atlaskit-mk-2
        let url =
            "https://bitbucket.org/atlassian/atlaskit-mk-2/src/master/build/docs/src/md/index.ts";
        let expected =
            "https://bitbucket.org/atlassian/atlaskit-mk-2/raw/master/build/docs/raw/md/index.ts";
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
    fn test_invalid_urls() {
        // Test invalid URL format
        assert!(convert_to_raw_url("not_a_url").is_err());

        // Test invalid GitHub URL
        assert!(convert_to_raw_url("https://github.com/user/repo").is_err());

        // Test invalid GitLab URL
        assert!(convert_to_raw_url("https://gitlab.com/user/repo/blob/master/file.rs").is_err());

        // Test invalid Playground URL (no gist parameter)
        assert!(convert_to_raw_url("https://play.rust-lang.org/?version=stable").is_err());
    }

    #[test]
    fn test_raw_url() {
        let url = "https://example.com/raw/file.rs";
        assert_eq!(convert_to_raw_url(url).unwrap(), url);
    }
}
