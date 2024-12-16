/// Demo determining at run-time whether an expression returns a unit value
/// so that it can be handled appropriately. When a code template this is
/// short and sweet but it has to be included in the templated and thus the
/// generated code, whereas using an AST is quite a mission but works with
/// any arbitrary snippet.
//# Purpose: Demo Rust's answer to dynamic typing.
use std::any::Any;

let displayable = |expr: &(dyn Any + 'static)| -> bool {
    // let expr_any: &dyn Any = &expr;
    !expr.is::<()>()
};

let expr = ();

let can_display = displayable(&expr);  // Box<dyn Any>::new(expr));
eprintln!("displayable(expr)={}", can_display);

if can_display {
    println!("{:?}", expr);
}

let expr: &'static str = "Hello world!";

let can_display = displayable(&expr);
eprintln!("displayable(expr)={}", can_display);

if can_display {
    println!("{:?}", expr);
}
