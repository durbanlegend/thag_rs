use std::error::Error;
use tinyget;

fn get_latest_commit(owner: &str, repo: &str, branch: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}?jq=",
        owner, repo, branch
    );

    let binding = tinyget::get(&url)
        .with_header("User-Agent", "rust-app")
        .send()?;
    let response = binding.as_str()?.as_bytes();

    // Parse the JSON response to extract the SHA
    let resp: serde_json::Value = serde_json::from_slice(response)?;
    let sha = resp["commit"]["sha"]
        .as_str()
        .ok_or("SHA not found")?
        .to_string();

    Ok(sha)
}

fn main() -> Result<(), Box<dyn Error>> {
    let sha = get_latest_commit("durbanlegend", "thag_rs", "develop")?;
    println!("Latest commit SHA: {}", sha);
    Ok(())
}
