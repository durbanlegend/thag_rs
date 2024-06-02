// #! /usr/bin/env /Users/donf/projects/rs-script/target/release/rs-script
/*[toml]
[dependencies]
itertools = "0.12.1"
*/
use itertools::Itertools;

let fib = |n: usize| -> usize {
    itertools::iterate((0, 1), |&(a, b)| (b, a + b))
        .take(n)
        .last()
        .unwrap()
        .0
};

println!("Enter a number from 0 to 91");
println!("Type lines of text at the prompt and hit Ctrl-{} on a new line when done", if cfg!(windows) {'Z'} else {'D'});

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer.trim_end()
    .parse()
    .expect("Can't parse input into a positive integer");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
