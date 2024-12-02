/*[toml]
[dependencies]
quote = "1.0.37"
syn = { version = "2.0.90", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

/// Demo of selectively modifying source code using `syn` and `quote`. This is from a solution posted by user Yandros on the Rust Playground
/// in answer to a question asked on the Rust users forum. The discussion and Playground link are to be found here:
/// https://users.rust-lang.org/t/writing-proc-macros-with-syn-is-there-a-way-to-visit-parts-of-the-ast-that-match-a-given-format/54733/4
/// (This content is dual-licensed under the MIT and Apache 2.0 licenses according to the Rust forum terms of service.)
//# Purpose: Demo programmatically modifying Rust source code using `syn` and `quote`.
//# Categories: AST, crates, technique
const INPUT_CODE: &str = stringify! {
    fn foobar() {
      do_something(1, 2, 3);
      do_something_blue(1, 2, 3);
      if some_condition() {
        if other_condition() {
          let a = get_value();
          let b = get_value_blue(a);
        }
      }
    }
};

fn main() {
    use ::quote::ToTokens;
    use ::syn::{visit_mut::*, *};

    let mut code: ItemFn = parse_str(INPUT_CODE).unwrap();

    struct AppendHelloToBlues;
    impl VisitMut for AppendHelloToBlues {
        fn visit_expr_call_mut(self: &'_ mut Self, call: &'_ mut ExprCall) {
            // 1 - subrecurse
            visit_expr_call_mut(self, call);
            // 2 - special case functions whose name ends in `_blue`
            if matches!(
                *call.func,
                Expr::Path(ExprPath { ref path, .. })
                if path.segments.last().unwrap().ident.to_string().ends_with("_blue")
            ) {
                call.args.push(parse_quote!("hello"));
            }
        }

        fn visit_expr_method_call_mut(self: &'_ mut Self, call: &'_ mut ExprMethodCall) {
            // 1 - subrecurse
            visit_expr_method_call_mut(self, call);
            // 2 - special case functions whose name ends in `_blue`
            if call.method.to_string().ends_with("_blue") {
                call.args.push(parse_quote!("hello"));
            }
        }
    }
    AppendHelloToBlues.visit_item_fn_mut(&mut code);
    println!("{}", code.into_token_stream());
}
