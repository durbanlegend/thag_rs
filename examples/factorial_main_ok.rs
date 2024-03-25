//: -s

use std::io::*;

fn main() -> Result<()> {
    let fac = |n: u128| -> u128 {
        if n == 0 {
            0
        } else {
            (1..=n).product()
        }
    };

    println!("Type a number from 0 to 34 at the prompt and hit Ctrl-D when done");
    println!(
        "(Larger numbers will overflow, because runner currently has to use u128 or less if you provide a full program).
For bigger numbers, use a crate like rug and a snippet."
    );

    let mut buffer = String::new();
    std::io::stdin().lock().read_to_string(&mut buffer)?;

    let n: u128 = buffer
        .trim_end()
        .parse()
        .expect("Can't parse input into a positive integer");

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
