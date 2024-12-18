use std::any::Any;

/// Proof of concept of distinguishing types that implement Display from those that implement
/// Debug, and printing using the Display or Debug trait accordingly. Worked out with recourse
/// to ChatGPT for suggestions and macro authoring.
//# Purpose: May be interesting or useful.
//# Categories: macros, technique, type_identification
macro_rules! generate_displayable_fn {
    ($func_name:ident, $($t:ty),*) => {
        fn $func_name(s: &(dyn Any + std::marker::Send)) -> bool {
            let types: &[&dyn Fn(&dyn Any) -> Option<String>] = &[
                $(
                    &|s| s.downcast_ref::<$t>().map(|x| format!("Displayable: '{x}'")),
                )*
            ];

            for checker in types {
                if let Some(output) = checker(s) {
                    println!("{}", output);
                    return true;
                }
            }

            println!("Not displayable...");
            false
        }
    };
}

// Use the macro to generate the function
generate_displayable_fn!(
    print_if_displayable,
    String,
    &str,
    bool,
    char,
    f32,
    f64,
    i128,
    i16,
    i32,
    i64,
    i8,
    isize,
    u128,
    u16,
    u32,
    u64,
    u8,
    usize
);

fn main() {
    let is_debuggable = |expr: &(dyn Any + 'static)| -> bool {
        let debuggable = !expr.is::<()>();
        println!("Debuggable? {debuggable}");
        debuggable
    };

    print_if_displayable(&"Hello, world!".to_string());
    print_if_displayable(&42);
    print_if_displayable(&"Hello!");
    print_if_displayable(&Some("Hello!"));
    print_if_displayable(&None::<&str>);
    print_if_displayable(&4.5_f64);

    let x = ();
    if !print_if_displayable(&x) && is_debuggable(&x) {
        println!("{x:?}");
    }
    println!();

    let x = "String".to_string();
    if !print_if_displayable(&x) && is_debuggable(&x) {
        println!("{x:?}");
    }
    println!();

    let x = "&str";
    if !print_if_displayable(&x) && is_debuggable(&x) {
        println!("{x:?}");
    }
    println!();

    let x = 0;
    if !print_if_displayable(&x) && is_debuggable(&x) {
        println!("{x:?}");
    }
    println!();

    let x = Some("Some(&str");
    if !print_if_displayable(&x) && is_debuggable(&x) {
        println!("{x:?}");
    }
}
