use std::process::{Command, Stdio};

fn convert_github_url_to_raw(url: &str) -> String {
    if url.contains("github.com") && url.contains("/blob/") {
        url.replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    } else {
        url.to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} [-s|-d] <github-url>", args[0]);
        std::process::exit(1);
    }

    let (flag, url) = if args.len() == 3 {
        let flag = match args[1].as_str() {
            "-s" | "-d" => &args[1],
            _ => {
                eprintln!("Invalid flag. Use -s or -d");
                std::process::exit(1);
            }
        };
        (flag.as_str(), &args[2])
    } else {
        ("-s", &args[1]) // default to -s if no flag provided
    };

    let raw_url = convert_github_url_to_raw(url);

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
    #[test]
    fn test_url_conversion() {
        let github_url = "https://github.com/durbanlegend/thag_rs/blob/master/demo/hello.rs";
        let expected =
            "https://raw.githubusercontent.com/durbanlegend/thag_rs/master/demo/hello.rs";
        assert_eq!(convert_github_url_to_raw(github_url), expected);
    }

    #[test]
    fn test_non_github_url() {
        let url = "https://example.com/script.rs";
        assert_eq!(convert_github_url_to_raw(url), url);
    }
}
