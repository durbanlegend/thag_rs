/// Demo determining at run-time whether an expression returns a unit value
/// so that it can be handled appropriately. When a code template this is
/// short and sweet but it has to be included in the templated and thus the
/// generated code, whereas using an AST is quite a mission but works with
/// any arbitrary snippet.
//# Purpose: Demo Rust's answer to dynamic typing.
use std::any::Any;
use std::fmt::Display;

let debuggable = |expr: &(dyn Any + 'static)| -> bool {
    !expr.is::<()>()
};

let is_string = |expr: &(dyn Any + 'static)| -> bool {
    expr.is::<&String>()
};

let displayable = |expr: &(dyn Any + 'static)| -> bool {
    expr.is::<&str>()
    || expr.is::<String>()
    || expr.is::<i8>()
    || expr.is::<i16>()
    || expr.is::<i32>()
    || expr.is::<i64>()
    || expr.is::<i128>()
    || expr.is::<isize>()
    || expr.is::<u8>()
    || expr.is::<u16>()
    || expr.is::<u32>()
    || expr.is::<u64>()
    || expr.is::<u128>()
    || expr.is::<usize>()
    || expr.is::<f32>()
    || expr.is::<f64>()
    || expr.is::<char>()
    || expr.is::<bool>()
};

let expr = ();
eprintln!("displayable(&expr)={}, debuggable(&expr)={}", displayable(&expr), debuggable(&expr));
if displayable(&expr) {
    println!("{expr:?}");
}

let expr = Some("Hello world!");
eprintln!("displayable(expr)={}", displayable(&expr));
if displayable(&expr) {
    let disp = expr.as_ref::<&dyn Display>();
    println!("{disp}");
}
if debuggable(&expr) {
    println!("{expr:?}");
}

let expr = String::from("Hello world!");

eprintln!("displayable(&expr)={}, debuggable(&expr)={}", displayable(&expr), debuggable(&expr));
if displayable(&expr) {
    println!("{expr}");
} else if debuggable(&expr) {
    println!("{expr:?}");
}
println!();

let expr = 'c';
eprintln!("displayable(expr)={}", displayable(&expr));
if displayable(&expr) {
    println!("{expr}");
} else if debuggable(&expr) {
    println!("{expr:?}");
}
println!();

let expr = "Hello world!";
eprintln!("displayable(expr)={}", displayable(&expr));
if displayable(&expr) {
    println!("{expr}");
} else if debuggable(&expr) {
    println!("{expr:?}");
}
println!();
