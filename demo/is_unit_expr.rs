/// Demo determining at run-time whether an expression returns a unit value
/// so that it can be handled appropriately.
///
/// `thag` needs to know whether an expression returns a unit type or a value
/// that we should display. When using a code template this approach using `Any`
/// is short and sweet, but it has to be included in the template and thus the
/// generated code, whereas the alternative of using an AST is quite a mission
/// but works with any arbitrary snippet and doesn't pollute the generated
/// source code, so `thag` went with the latter.
//# Purpose: Demo Rust's answer to dynamic typing.
//# Categories: exploration, type_identification, technique
use std::any::Any;

let is_unit_expr = |expr: &dyn Any| {
    expr.is::<()>()
};

let expr = ();
let is_unit = is_unit_expr(&expr);
eprintln!("expr=`{expr:?}`, is_unit={is_unit}");

if is_unit {
    println!("Printing unit type as `Display`: expr={expr:?}");
    println!();
}

let expr = 2 + 3;

let is_unit = is_unit_expr(&expr);

eprintln!("expr=`{expr:?}`, is_unit={is_unit}");

if is_unit {
    println!("Printing unit type as `Display`: expr={expr}");
}
