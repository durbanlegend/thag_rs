use std::process::Command;
use std::str;

fn get_latest_commit(
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/{}/branches/{}", owner, repo, branch),
            "--jq",
            ".commit.sha",
        ])
        .output()?;

    if !output.status.success() {
        return Err(str::from_utf8(&output.stderr)?.into());
    }

    Ok(str::from_utf8(&output.stdout)?.trim().to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sha = get_latest_commit("durbanlegend", "thag_rs", "develop")?;
    println!("Latest commit SHA: {}", sha);
    Ok(())
}
