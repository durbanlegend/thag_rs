//! Demo: Adding a string to an iterator over Vec<String>
//! This example shows different ways to append a string to a collection
//! of strings using iterators and the `chain()` method.

fn main() {
    println!("=== String Iterator Chaining Demo ===\n");

    // Create a vector of strings
    let names = vec![
        String::from("Alice"),
        String::from("Bob"),
        String::from("Charlie"),
    ];
    println!("Original names: {:?}", names);

    // Method 1: Chain with Some(String)
    // This demonstrates chaining a Some(String) to an iterator
    let extra_name = String::from("David");
    let extended_names: Vec<String> = names
        .iter()
        .cloned() // We clone because we want to reuse 'names' later
        .chain(Some(extra_name))
        .collect();

    println!("After chaining Some(String): {:?}", extended_names);

    // Method 2: Using references with chain
    // Here we chain a reference to a static string
    let static_name: &'static str = "Eve";
    let names_with_static: Vec<String> = names
        .iter()
        .cloned()
        .chain(std::iter::once(String::from(static_name)))
        .collect();

    println!("After chaining static str: {:?}", names_with_static);

    // Method 3: Working with string slices
    // Converting to a collection of &str with a static str appended
    let names_with_slices: Vec<&str> = names
        .iter()
        .map(|s| s.as_str()) // Convert String -> &str
        .chain(std::iter::once("Frank")) // Add a string literal (which is &'static str)
        .collect();

    println!("Collection with string slices: {:?}", names_with_slices);

    // Method 4: String to actual 'static str via leaking
    // THIS LEAKS MEMORY - only use when appropriate!
    // Create leaked strings as &'static mut str and convert them
    let leaked: Vec<&'static str> = names
        .iter()
        .map(|s| {
            // Box::leak returns &'static mut str, so we need to cast to &'static str
            let static_mut_str: &'static mut str = Box::leak(s.clone().into_boxed_str());
            static_mut_str as &'static str // Cast mutable reference to immutable
        })
        .collect();

    // Then add one more in a separate step
    let leaked_with_grace: Vec<&'static str> =
        leaked.into_iter().chain(std::iter::once("Grace")).collect();

    println!(
        "Names with leaked memory (static lifetime): {:?}",
        leaked_with_grace
    );
    println!("(Note: the above example leaks memory - only use Box::leak when appropriate)");

    println!(
        "Names with leaked memory (static lifetime): {:?}",
        names
            .iter()
            .map(|s| {
                // Box::leak returns &'static mut str, so we need to cast to &'static str
                let static_mut_str: &'static mut str = Box::leak(s.clone().into_boxed_str());
                static_mut_str as &'static str // Cast mutable reference to immutable
            })
            .collect::<Vec<_>>()
            .into_iter()
            .chain(std::iter::once("Grace"))
            .collect::<Vec<_>>()
    );
}
