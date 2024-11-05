/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2.0.87", features = ["full"] }
*/

/// Typical basic (runtime) solution to expression type identification. See also `demo/determine_if_known_type_trait.rs`
/// for what may be a better (compile-time) solution depending on your use case.
//# Purpose: Demo of runtime type identification.
use quote::quote;
use syn::Expr;

fn type_of<T>(_x: &T) -> String {
    std::any::type_name::<T>().to_string()
}

fn main() {
    let s = "Hello";
    let i = 42;

    println!("Type of {} is {}", stringify!(&s), type_of(&s)); // &str
    println!("Type of {} is {}", stringify!(&i), type_of(&i)); // i32
    println!("Type of {} is {}", stringify!(&main), type_of(&main)); // playground::main
    println!(
        "Type of {} is {}",
        stringify!(&type_of::<i32>),
        type_of(&type_of::<i32>)
    ); // playground::print_type_of<i32>
    println!(
        "Type of {} is {}",
        stringify!(&{ || "Hi!" }),
        type_of(&{ || "Hi!" })
    ); // playground::main::{{closure}}
    println!("Type of {} is {}", stringify!(&(2 + 3)), type_of(&(2 + 3)));
    println!(
        "Type of {} is {}",
        stringify!(&syn::parse2::<Expr>(quote!({ 3 }).into())),
        type_of(&syn::parse2::<Expr>(quote!({ 3 }).into()))
    );
    println!(
        "Type of {} is {}",
        stringify!(&quote!(())),
        type_of(&quote!(()))
    );
    println!("Type of {} is {}", stringify!(&1), type_of(&1));
    println!("Type of {} is {}", stringify!(&&1), type_of(&&1));
    println!("Type of {} is {}", stringify!(&&&1), type_of(&&&1));
    println!("Type of {} is {}", stringify!(&mut 1), type_of(&mut 1));
    println!("Type of {} is {}", stringify!(&&mut 1), type_of(&&mut 1));
    println!("Type of {} is {}", stringify!(&mut &1), type_of(&mut &1));
    println!("Type of {} is {}", stringify!(&1.0), type_of(&1.0));
    println!("Type of {} is {}", stringify!(&"abc"), type_of(&"abc"));
    println!("Type of {} is {}", stringify!(&&"abc"), type_of(&&"abc"));
    println!(
        "Type of {} is {}",
        stringify!(&String::from("abc")),
        type_of(&String::from("abc"))
    );
    println!(
        "Type of {} is {}",
        stringify!(&vec![1, 2, 3]),
        type_of(&vec![1, 2, 3])
    );
    println!(
        "Type of {} is {}",
        stringify!(&syn::parse2::<Expr>(quote!({ 3 }).into())),
        type_of(&syn::parse2::<Expr>(quote!({ 3 }).into()))
    );
}
