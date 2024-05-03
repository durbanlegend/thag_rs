/// This works well, but author recommends trait alternative
/// https://stackoverflow.com/questions/34214136/how-do-i-match-the-type-of-an-expression-in-a-rust-macro/34214916#34214916

struct Attribute<T> {
    value: T,
}

impl Attribute<u8> {
    fn call(&self) {
        println!("{} is a u8", self.value);
    }
}

impl Attribute<u32> {
    fn call(&self) {
        println!("{} is a u32", self.value);
    }
}

impl Attribute<f64> {
    fn call(&self) {
        println!("{} is a f64", self.value);
    }
}

impl Attribute<String> {
    fn call(&self) {
        println!("{} is a string", self.value);
    }
}

impl Attribute<&str> {
    fn call(&self) {
        println!("{} is an &str", self.value);
    }
}

impl Attribute<()> {
    fn call(&self) {
        println!("{:?} is a unit type", self.value);
    }
}

macro_rules! attribute {
    ( $e:expr ) => {
        Attribute { value: $e }.call();
    };
}

fn main() {
    let a = Attribute { value: 5_u32 };
    a.call();

    Attribute { value: 6.5 }.call();

    attribute!(5_u32);
    attribute!(6.5);
    attribute!("Hello World!".to_string());
    attribute!("Hello World!");
    attribute!(());
}
