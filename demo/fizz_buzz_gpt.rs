//: GPT-generated fizz-buzz example.
//# Purpose: Demo running random snippets in thag_rs, also AI and the art of delegation ;)
//# Categories: educational, technique
for i in 1..=100 {
    match (i % 3 == 0, i % 5 == 0) {
        (true, true) => println!("FizzBuzz"),
        (true, false) => println!("Fizz"),
        (false, true) => println!("Buzz"),
        (false, false) => println!("{}", i),
    }
}
