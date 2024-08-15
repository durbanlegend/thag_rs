/// A fun example from Programming Rust by Jim Blandy and Jason Orendorff (O’Reilly).
/// Copyright 2018 Jim Blandy and Jason Orendorff, 978-1-491-92728-1.
/// Described by the authors as "a really gratuitous use of iterators".
//# Purpose: Demo using `thag_rs` to try out random code snippets ... also iterators.
use std::iter::{once, repeat};

let fizzes = repeat("").take(2).chain(once("fizz")).cycle();
let buzzes = repeat("").take(4).chain(once("buzz")).cycle();
let fizzes_buzzes = fizzes.zip(buzzes);

let fizz_buzz = (1..100).zip(fizzes_buzzes).map(|tuple| match tuple {
    (i, ("", "")) => i.to_string(),
    (_, (fizz, buzz)) => format!("{}{}", fizz, buzz),
});

for line in fizz_buzz {
    println!("{}", line);
}
