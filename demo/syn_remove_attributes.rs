/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.72", features = ["fold", "extra-traits", "full", "parsing", "visit-mut"] }
*/

/// Prototype of removing an inner attribute (`#![...]`) from a syntax tree. Requires the `visit-mut'
/// feature of `syn`.
//# Purpose: Demonstrate making changes to a `syn` AST.
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{AttrStyle, ExprBlock};

struct RemoveInnerAttributes;

impl VisitMut for RemoveInnerAttributes {
    fn visit_expr_block_mut(&mut self, expr_block: &mut ExprBlock) {
        // Filter out inner attributes
        expr_block
            .attrs
            .retain(|attr| attr.style != AttrStyle::Inner(syn::token::Not::default()));

        // Continue visiting the rest of the expression block
        visit_mut::visit_expr_block_mut(self, expr_block);
    }
}

fn main() {
    let source = r#"{
        #![feature(duration_constructors)]
        use std::time::Duration;
        Duration::from_days(10);
    }"#;

    // Parse the source code into an expression block
    let mut expr_block: ExprBlock = syn::parse_str(source).expect("Failed to parse source");

    // Apply the RemoveInnerAttributes visitor to the expression block
    RemoveInnerAttributes.visit_expr_block_mut(&mut expr_block);

    // Print the modified expression block
    println!("{expr_block:#?}");

    // Convert back using quote:
    println!("{}", quote!(#expr_block));
}
