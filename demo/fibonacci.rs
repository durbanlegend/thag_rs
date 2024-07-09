#! /usr/bin/env /Users/donf/projects/rs-script/target/debug/rs_script
/*[toml]
[dependencies]
itertools = "0.12.1"
*/
use itertools::iterate;

/// Fast non-recursive fibonacci sequence calculation. Can't recall the exact source
/// but see for example https://users.rust-lang.org/t/fibonacci-sequence-fun/77495
/// for a variety of alternative approaches.
let fib = |n: usize| -> usize {
    match n {
        0 => 0_usize,
        1 => 1_usize,
        _ => {
            iterate((0, 1), |&(a, b)| (b, a + b))
                .take(n)
                .last()
                .unwrap()
                .1
        }
    }
};

println!("Enter a number from 0 to 92");
println!("Type lines of text at the prompt and hit Ctrl-{} on a new line when done", if cfg!(windows) {'Z'} else {'D'});

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer.trim_end()
    .parse()
    .expect("Can't parse input into a positive integer");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
