use itertools::iterate;

fn fib(n: usize) -> usize {
    match n {
        0 => 0_usize,
        1 => 1_usize,
        _ => {
            iterate((0, 1), |&(a, b)| (b, a + b))
                .nth(n - 1)
                .unwrap()
                .1
        }
    }
}

println!("Enter a number from 0 to 92");
println!("Type lines of text at the prompt and hit Ctrl-{} on a new line when done", if cfg!(windows) {'Z'} else {'D'});

let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer)?;

let n: usize = buffer.trim_end()
    .parse()
    .expect("Can't parse input into a positive integer");

let f = fib(n);
println!("Number {n} in the Fibonacci sequence is {f}");
