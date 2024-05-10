/*[toml]
[dependencies]
quote = "1.0.36"
*/
use quote::quote;

fn print_type_of<T>(_x: &T) {
    println!("Type is {}", std::any::type_name::<T>())
}

fn main() {
    let s = "Hello";
    let i = 42;

    print_type_of(&s); // &str
    print_type_of(&i); // i32
    print_type_of(&main); // playground::main
    print_type_of(&print_type_of::<i32>); // playground::print_type_of<i32>
    print_type_of(&{ || "Hi!" }); // playground::main::{{closure}}
    print_type_of(&(2 + 3));
    print_type_of(&quote!({ 3 }));
}
