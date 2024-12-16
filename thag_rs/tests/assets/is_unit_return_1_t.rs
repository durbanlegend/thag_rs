let mut i = 0;
let x = loop {
    i += 1;
    match i % 3 {
        0 => "scissors",  // panic!("Oh noes!")
        1 => "rock",
        2 => return "paper",
        _ => panic!("lolwut?"),
    };
};
x
