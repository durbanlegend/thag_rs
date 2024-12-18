/// Demo determining at run-time whether an expression returns a unit value
/// so that it can be handled appropriately. When using a code template this is
/// short and sweet, but it has to be included in the template and thus the
/// generated code, whereas using an AST is quite a mission but works with
/// any arbitrary snippet and doesn't pollute the generated source code.
//# Purpose: Demo Rust's answer to dynamic typing.
//# Categories: type_identification, technique
use std::any::Any;

let expr = ();

let display = {
    let expr_any: &dyn Any = &expr;
    !expr_any.is::<()>()
};

eprintln!("display={display}");

if display {
    println!("{:?}", expr);
}

let expr = 2 + 3;

let display = {
    let expr_any: &dyn Any = &expr;
    !expr_any.is::<()>()
};

eprintln!("display={display}");

if display {
    println!("{:?}", expr);
}
