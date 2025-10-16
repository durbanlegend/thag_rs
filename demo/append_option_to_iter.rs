/// Demo: Optionally append one item to an iterator.
/// The trick is that `Option` implements the `IntoIterator` trait.
//# Purpose: demo a handy trick.
//# Categories: learning, technique
fn main() {
    println!(
        "[1, 2, 3].iter().chain(Some(&4)).collect::<Vec<_>>() = {:?}",
        [1, 2, 3].iter().chain(Some(&4)).collect::<Vec<_>>()
    );
    println!();
    println!(
        "[1, 2, 3].iter().chain(None).collect::<Vec<_>>() = {:?}",
        [1, 2, 3].iter().chain(None).collect::<Vec<_>>()
    );
}
