use rug::Integer;
use std::error::Error;
use std::io;
use std::io::Read;

fn main() -> Result<(), Box<dyn Error>> {
    let fac = |n: usize| -> Integer {
        if n == 0 {
            Integer::from(0_usize)
        } else {
            (1..=n).map(Integer::from).product()
        }
    };

    println!("Type lines of text at the prompt and hit Ctrl-D when done");

    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;

    let n: usize = buffer
        .trim_end()
        .parse()
        .expect("Can't parse input into a positive integer");

    println!("fac({n}) = {}", fac(n));
    Ok(())
}
