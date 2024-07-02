match 34_usize % 3 {
    0 => "scissors",  // panic!("Oh noes!")
    1 => "rock",
    2 => "paper",
    _ => panic!("lolwut?"),
}
