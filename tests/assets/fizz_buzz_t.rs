for i in 1..=100 {
    match (i % 3 == 0, i % 5 == 0) {
        (true, true) => println!("FizzBuzz"),
        (true, false) => println!("Fizz"),
        (false, true) => println!("Buzz"),
        (false, false) => println!("{}", i),
    }
}
