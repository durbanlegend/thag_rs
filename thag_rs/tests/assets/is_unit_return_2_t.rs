let rock = "rock";
match 34_usize % 3 {
    0 => {
        println!("Hello!");
        "paper"
    }
    1 => "scissors",  // panic!("Oh noes!")
    2 => rock,
    _ => panic!("lolwut?"),
}
