use std::process::Command;
use std::str;

fn get_latest_commit(
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        owner, repo, branch
    );

    let output = Command::new("curl")
        .args([
            "-s", // silent mode
            "-H",
            "Accept: application/vnd.github.v3+json",
            "-H",
            "User-Agent: curl",
            &url,
        ])
        .output()?;

    if !output.status.success() {
        return Err(str::from_utf8(&output.stderr)?.into());
    }

    // Parse the JSON response to extract the SHA
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let sha = response["commit"]["sha"]
        .as_str()
        .ok_or("SHA not found")?
        .to_string();

    Ok(sha)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sha = get_latest_commit("durbanlegend", "thag_rs", "develop")?;
    println!("Latest commit SHA: {}", sha);
    Ok(())
}
