/*[toml]
[dependencies]
chrono = "0.4.38"
*/

/// Simple time demo pasted directly from Rust cookbook. Run without -q to show how
/// `thag_rs` will find the missing `chrono` manifest entry and display a specimen
/// toml block you can paste in at the top of the script.
//# Purpose: Demo cut and paste from a web source with Cargo search and specimen toml block generation.
//# Categories: basic
use chrono::{DateTime, Utc};

fn main() {
    let now: DateTime<Utc> = Utc::now();

    println!("UTC now is: {}", now);
    println!("UTC now in RFC 2822 is: {}", now.to_rfc2822());
    println!("UTC now in RFC 3339 is: {}", now.to_rfc3339());
    println!(
        "UTC now in a custom format is: {}",
        now.format("%a %b %e %T %Y")
    );
}
