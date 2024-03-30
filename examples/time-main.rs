//! [dependencies]
//! time = "0.1.25"

//  use time::*;

fn main() {
    println!("{}", time::now().rfc822z());
}
