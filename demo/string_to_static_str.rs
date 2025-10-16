/// Demo: Convert a `String` to a `&'static str` at runtime, then do
/// the same for a whole vector of `String`s.
///
/// This should only be used when it's appropriate for the string
/// reference to remain allocated for the life of the program.
//# Purpose: demo a handy trick.
//# Categories: learning, technique
fn main() {
    let string = String::from("Abracadabra");

    println!(
        "Original String: {string}; as `&'static str` with leaked memory (static lifetime): {}",
        Box::leak(string.clone().into_boxed_str())
    );

    // Create a vector of strings
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];
    println!("Original names: {:?}", names);

    // THIS LEAKS MEMORY - only use when appropriate!
    let leaked_strings: Vec<&'static str> = names
        .iter()
        .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str)
        .chain(Some("Grace"))
        .chain(std::iter::once("Henry"))
        .collect();

    println!(
        "Names with leaked memory (static lifetime): {:?}",
        leaked_strings
    );
    // println!("(Note: the above example leaks memory - only use Box::leak when appropriate)");
}
