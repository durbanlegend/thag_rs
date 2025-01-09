pub fn get(i: usize) -> String {
    let book_slices: &[&String] = &[&"IT".to_string(), &"Harry Potter".to_string()];

    // for book in book_slices {
    //     println!("{}", book);  // Prints the string the reference points to
    // }

    *book_slices[i]
}

let i = 1;
println!("entry {i} = {}", get(i));
