// let x = 34_usize % 3;
// if x == 0 {
//     panic ("Oh noes!");
// } else if x == 1 {
//     "rock"
// } else if x == 2 {
//     "paper"
// } else if x == 3 {
//     "scissors"
// } else { panic!("lolwut?"); }

let x = 34_usize % 3;
if x == 0 {
    panic!("Oh noes!");
} else if x == 1 {
    "rock"
} else if x == 2 {
    "paper"
} else if x == 3 {
    "scissors"
} else {
    panic!("lolwut?");
}
