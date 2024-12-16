use std::any::Any;
use std::fmt::Display;

let is_debuggable = |expr: &(dyn Any + 'static)| !expr.is::<()>();

// fn print_if_debuggable(expr: &(dyn Any + 'static)) {
//     if expr.is::<()>() {
//         println!("Unit type, can't print");
//     } else {
//         println!
//     }
// }

fn print_non_unit(s: &(dyn Any + Send)) {
    if let Some(x) = s.downcast_ref::<String>() {
        println!("Displayable: '{x}'");
    } else if let Some(x) = s.downcast_ref::<&str>() {
        println!("Displayable: '{x}'");
    } else if let Some(x) = s.downcast_ref::<()>() {
        println!("Unit type, can't print");
    } else {
        println!("Assume debuggable: '{s:?}'");
    }
}

let x = ();
print_non_unit(&x);
println!();

let x = "String".to_string();
print_non_unit(&x);
println!();

let x = "&str";
print_non_unit(&x);
println!();

let x = 0;
print_non_unit(&x);
println!();

let x = Some("Some(&str");
print_non_unit(&x);
println!();
