fn main() {
    let data = vec!["apple", "banana", "cherry", "damson"];
    let mut flag = false;

    let v = data
        .iter()
        .peekable()
        .skip_while(|element| {
            if *element != &"banana" {
                if *element == &"apple" {
                    // println!("Element == 'apple': {}", element);
                    flag = true;
                }
                true // Skip elements until "banana" is found
            } else {
                // println!("Element != 'apple': {}", element);
                false // Stop skipping elements
            }
        })
        .collect::<Vec<_>>();

    println!("Vector after skipping: {:?}", v);

    if flag {
        println!("Flag is true");
    } else {
        println!("The iterator does not contain 'banana'.");
    }
}
