/*[toml]
[dependencies]
dashu = "0.4.2"
*/

/// Use a trait to determine the type of an expression at compile time, provided all cases are known in advance.
/// Most upvoted and recommended answer on Stack Overflow page:
/// https://stackoverflow.com/questions/34214136/how-do-i-match-the-type-of-an-expression-in-a-rust-macro/34214916#34214916
/// seems to work very well provided all the types encountered are anticipated.
//# Purpose: Demo expression type deteermination for static dispatch.
use dashu::integer::IBig;

trait Attribute {
    fn process(&self);
}

impl Attribute for f32 {
    fn process(&self) {
        println!("{} is a f32", self);
    }
}

impl Attribute for u8 {
    fn process(&self) {
        println!("{} is a u8", self);
    }
}

impl Attribute for u32 {
    fn process(&self) {
        println!("{} is a u32", self);
    }
}

impl Attribute for i32 {
    fn process(&self) {
        println!("{} is an i32", self);
    }
}

impl Attribute for i64 {
    fn process(&self) {
        println!("{} is an i64", self);
    }
}

impl Attribute for String {
    fn process(&self) {
        println!("{:?} is a String", self);
    }
}

impl Attribute for &String {
    fn process(&self) {
        println!("{:?} is an &String", self);
    }
}

impl Attribute for &str {
    fn process(&self) {
        println!("{:?} is an &str", self);
    }
}

impl Attribute for () {
    fn process(&self) {
        println!("{:?} is a unit type", self);
    }
}

// impl Attribute for TypeNever {
//     fn process(&self) {
//         println!("{} is a never type", self);
//     }
// }

impl<T: std::fmt::Debug> Attribute for Option<T> {
    fn process(&self) {
        if let Some(value) = self {
            println!("Option({:?}) is an Option type", value);
        } else {
            println!("{:?} is an Option type", self);
        }
    }
}

impl<T: std::fmt::Debug, E: std::fmt::Debug> Attribute for Result<T, E> {
    fn process(&self) {
        if let Ok(value) = self {
            println!("Ok({:?}) is a Result type", value);
        } else {
            println!("{:?} is a Result type", self);
        }
    }
}

impl Attribute for IBig {
    fn process(&self) {
        println!("{} is a dashu::integer::IBig type", self);
    }
}

macro_rules! attribute {
    ($e:expr) => {
        Attribute::process(&$e)
    };
}

fn main() {
    println!("Script for determining expression types");
    attribute!(2 + 2);
    attribute!(255);
    attribute!(-127);
    attribute!(5_u32);
    attribute!(6.5);
    attribute!(String::from("Hello World!").as_str());
    attribute!(&String::from("Hello World!"));
    attribute!("Hello World!");
    attribute!(());
    attribute!(Some("Hello World!"));
    attribute!(None::<String>);
    attribute!(Ok::<&str, &str>("Hello World!"));
    attribute!(Err::<&str, &str>("Bad thing happened"));
    attribute!(Ok::<Option<&str>, &str>(Some("Hello World!")));
    attribute!(IBig::from(0_usize));
}
