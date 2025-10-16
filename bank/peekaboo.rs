#![allow(clippy::cargo_common_metadata)]

fn main() {
    let data = ["apple", "banana", "cherry", "damson"];

    let mut iter = data
        .iter()
        .skip_while(|element| *element != &"banana" && *element == &"apple")
        .peekable();

    assert!(iter.peek() == Some(&&"banana"));

    let v = iter.copied().collect::<Vec<_>>();

    println!("Vector after skipping: {v:?}");
}
