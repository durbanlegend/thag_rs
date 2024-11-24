/*[toml]
[dependencies]
tinyget = { version = "1.0.2", features = ["https"] }
*/
use std::error::Error;
use tinyget;

fn main() -> Result<(), Box<dyn Error>> {
    let url =
        "https://github.com/near-cli-rs/interactive-clap/blob/master/examples/advanced_struct.rs";
    let raw_url = convert_github_url_to_raw(&url)?;
    let response = tinyget::get(raw_url).send()?;
    let hello = response.as_str()?;
    println!("{}", hello);
    Ok(())
}

fn convert_github_url_to_raw(url_str: &str) -> Result<String, Box<dyn Error>> {
    // let url_str = url.as_str();
    if url_str.contains("github.com") {
        // Convert github.com URLs to raw.githubusercontent.com
        let raw_url = url_str
            .replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/");
        Ok(raw_url)
    } else {
        Ok(url_str.to_string())
    }
}
