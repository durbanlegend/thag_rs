/*[toml]
[dependencies]
proc-macro2 = "1.0.83"
quote = "1.0.36"
syn = { version = "2.0.60", features = ["full"] }
*/
use quote::quote;
use syn::Expr;

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
    print_type_of(&syn::parse2::<Expr>(quote!({ 3 }).into()));
    print_type_of(&quote!(()));
}
