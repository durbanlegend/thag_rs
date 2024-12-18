/*[toml]
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0.132"
# similar = { version ="2.6.0", features = ["text", "inline", "bytes"] }
similar = { path="/Users/donf/projects/similar/", features = ["text", "inline", "bytes", "serde"] }
*/

use serde_json;
use similar::TextDiff;

fn main() {
    let diff = TextDiff::from_lines(
        "Hello World\nThis is the second line.\nThis is the third.",
        "Hallo Welt\nThis is the second line.\nThis is life.\nMoar and more",
    );

    let all_changes = diff
        .ops()
        .iter()
        .flat_map(|op| diff.iter_changes(op))
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string_pretty(&all_changes).unwrap());
}
