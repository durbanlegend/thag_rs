#![allow(unused_variables)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, visit_mut::VisitMut, Expr, Ident, ItemConst, ItemStruct, Lit, Meta};

// Custom visitor for AST manipulation
struct MappingsVisitor<'a> {
    deletions: &'a Vec<String>,
}

// impl<'a> VisitMut for MappingsVisitor<'a> {
//     fn visit_expr_array_mut(&mut self, array: &mut syn::ExprArray) {
//         // We want to modify the array in place by removing items
//         array.elems.retain(|elem| {
//             // Check if the element is in the deletions
//             match elem {
//                 Expr::Tuple(tuple) => {
//                     // Assuming the key binding is stored in the second element of the tuple (index 1)
//                     if let Expr::Lit(lit) = &tuple.elems[1] {
//                         if let Lit::Str(key) = &lit.lit {
//                             // If the key is in the deletions list, remove it
//                             !self.deletions.contains(&key.value())
//                         } else {
//                             true
//                         }
//                     } else {
//                         true
//                     }
//                 }
//                 _ => true,
//             }
//         });
//         // Continue walking the rest of the array
//         syn::visit_mut::visit_expr_array_mut(self, array);
//     }
// }

pub fn use_mappings_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the struct that the macro is applied to
    // let input = parse_macro_input!(item as ItemStruct);
    let args = parse_macro_input!(attr with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    // let attr_meta = parse_macro_input!(attr as Meta);
    eprintln!("args={args:#?}");

    let mut base_ident: Option<Expr> = None;
    let mut additions_ident: Option<Expr> = None;
    let mut deletions_ident: Option<Vec<String>> = None;

    eprintln!("args={args:#?}");

    // // Handle the attribute input to extract base, additions, and deletions
    // if let Meta::List(meta_list) = attr_meta {
    //     for meta in meta_list.nested {
    //         if let NestedMeta::Meta(Meta::NameValue(nv)) = meta {
    //             let key = nv.path.get_ident().unwrap().to_string();
    //             if key == "base" {
    //                 if let Lit::Str(litstr) = &nv.value {
    //                     // Parse the base as an Expr (in this case, as an array)
    //                     base_const = Some(syn::parse_str(&litstr.value()).unwrap());
    //                 }
    //             } else if key == "additions" {
    //                 if let Lit::Str(litstr) = &nv.value {
    //                     // Parse the additions as an Expr
    //                     additions_const = Some(syn::parse_str(&litstr.value()).unwrap());
    //                 }
    //             } else if key == "deletions" {
    //                 if let Lit::Str(litstr) = &nv.value {
    //                     // Parse deletions as a list of strings
    //                     let deletions: Vec<String> = litstr
    //                         .value()
    //                         .split(',')
    //                         .map(|s| s.trim().to_string())
    //                         .collect();
    //                     deletions_list = Some(deletions);
    //                 }
    //             }
    //         }
    //     }
    // }

    let base_expr = base_ident.expect("Base mappings not provided");
    let additions_expr = additions_ident.expect("Additions mappings not provided");
    let deletions_vec = deletions_ident.expect("Deletions list not provided");

    // Use `VisitMut` to modify the base mappings by removing deletions
    let mut visitor = MappingsVisitor {
        deletions: &deletions_vec,
    };
    let mut base_ast: Expr = syn::parse_quote! { #base_expr };
    // visitor.visit_expr_mut(&mut base_ast);

    // Generate the final code by combining the modified base and the additions
    let expanded = quote! {
        const MAPPINGS: [(i32, &str, &str)] = #base_ast;

        const ADDITIONS: [(i32, &str, &str)] = #additions_expr;

        const FINAL_MAPPINGS: [(i32, &str, &str)] = {
            let mut mappings = MAPPINGS.to_vec();
            mappings.extend(ADDITIONS.iter().cloned());
            mappings
        };
    };

    TokenStream::from(expanded)
}
