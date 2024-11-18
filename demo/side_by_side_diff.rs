/*[toml]
[dependencies]
side-by-side-diff = "0.1.2"
*/

/// Published example from `side-by-side-diff` crate.
//# Purpose: Explore integrated side by side diffs.
use side_by_side_diff::create_side_by_side_diff;

fn main() {
    let diff = create_side_by_side_diff("aaa\niii\nuuu", "aaa\nii\nuuu", 20);
    println!("{diff}");
}
