use std::any::Any;
use std::fmt::{Debug, Display};

fn print_if_displayable<T: Debug + 'static>(value: &T) {
    if let Some(displayable) = value_as_displayable(value) {
        println!("{}", displayable);
    } else {
        println!("{:?}", value);
    }
}

fn value_as_displayable<T: Debug + 'static>(value: &T) -> Option<&dyn Display> {
    // This function will return `Some(&value)` if T implements `Display`,
    // otherwise, it returns `None`.
    (value as &dyn Any)
        .downcast_ref::<&dyn Display>()
        .map(|d| *d)
}

fn main() {
    let my_string = "Hello, world!";
    let my_number = 42;

    print_if_displayable(&my_string);
    print_if_displayable(&my_number);
}
