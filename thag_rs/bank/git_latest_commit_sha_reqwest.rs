/*[toml]
[dependencies]
reqwest = { version = "0.12.9", features = ["json"] }
serde_json = "1.0.133"
tokio = { version = "1.41.1", features = ["full"] }
*/

use reqwest;
use serde_json::Value;

async fn get_latest_commit(
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        owner, repo, branch
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-app")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let sha = response["commit"]["sha"]
        .as_str()
        .ok_or("SHA not found")?
        .to_string();

    Ok(sha)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sha = get_latest_commit("durbanlegend", "thag_rs", "develop").await?;
    println!("Latest commit SHA: {}", sha);
    Ok(())
}
