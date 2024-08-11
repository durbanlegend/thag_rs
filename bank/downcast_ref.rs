use std::any::Any;
use std::fmt::Display;

fn print_if_string(s: &(dyn Any + Send)) -> bool {
    if let Some(x) = s.downcast_ref::<String>() {
        println!("String: '{x}'");
        true
    } else if let Some(x) = s.downcast_ref::<&str>() {
        println!("&str: '{x}'");
        true
    } else {
        println!("Not a string...");
        false
    }
}

let is_debuggable = |expr: &(dyn Any + 'static)| -> bool {
    let debuggable = !expr.is::<()>();
    println!("Debuggable? {debuggable}");
    debuggable
};

let x = ();
if !print_if_string(&x) && is_debuggable(&x) {
    println!("{x:?}");
}
println!();

let x = "String".to_string();
if !print_if_string(&x) && is_debuggable(&x) {
    println!("{x:?}");
}
println!();


let x = "&str";
if !print_if_string(&x) && is_debuggable(&x) {
    println!("{x:?}");
}
println!();

let x = 0;
if !print_if_string(&x) && is_debuggable(&x) {
    println!("{x:?}");
}
println!();

let x = Some("Some(&str)");
if !print_if_string(&x) && is_debuggable(&x) {
    println!("{x:?}");
}
println!();
