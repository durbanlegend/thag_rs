#![allow(unused_variables)]
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::{visit_mut::VisitMut, Expr, ItemStruct, Lit, Meta};

// Custom visitor for AST manipulation
#[allow(dead_code)]
struct MappingsVisitor<'a> {
    deletions: &'a Vec<String>,
}

impl<'a> VisitMut for MappingsVisitor<'a> {
    fn visit_expr_array_mut(&mut self, array: &mut syn::ExprArray) {
        // Collect elements into a Vec, apply retain, then reconstruct the array
        let mut elems: Vec<Expr> = array.elems.iter().cloned().collect();

        elems.retain(|elem| match elem {
            Expr::Tuple(tuple) => {
                if let Expr::Lit(lit) = &tuple.elems[1] {
                    if let Lit::Str(key) = &lit.lit {
                        !self.deletions.contains(&key.value())
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => true,
        });

        // Reassign the modified elements back into `array.elems`
        array.elems = elems
            .into_iter()
            .collect::<syn::punctuated::Punctuated<_, _>>();
        syn::visit_mut::visit_expr_array_mut(self, array);
    }
}

pub fn use_mappings_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute input
    let args = parse_macro_input!(attr with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    println!("args={args:#?}");
    let input_struct = parse_macro_input!(input as ItemStruct);

    let mut base: Option<Expr> = None;
    let mut additions: Option<Expr> = None;
    let mut deletions: Option<Expr> = None;
    // let mut deletions: Option<Vec<String>> = None;

    // Extract each argument by matching its identifier (base, additions, deletions)
    for arg in &args {
        match arg {
            Meta::NameValue(nv) => {
                let ident = nv.path.get_ident().unwrap().to_string();
                if ident == "base" {
                    base = Some(nv.value.clone());
                } else if ident == "additions" {
                    additions = Some(nv.value.clone());
                } else if ident == "deletions" {
                    deletions = Some(nv.value.clone());
                }
            }
            _ => panic!("Expected name-value pair for base, additions, and deletions"),
        }
    }

    eprintln!("base={base:#?}");
    println!("additions={additions:#?}");

    // let mut base_ident: Option<Expr> = None;
    // let mut additions_ident: Option<Expr> = None;
    // let mut deletions_ident: Option<Vec<String>> = None;

    // for meta in args {
    //     if let Meta::NameValue(nv) = meta {
    //         let key = nv.path.get_ident().unwrap().to_string();

    //         if key == "base" {
    //             // Check if the value is an expression literal and downcast it to a string
    //             if let Expr::Lit(expr_lit) = &nv.value {
    //                 // eprintln!("expr_lit={expr_lit:?}");
    //                 if let Lit::Str(litstr) = &expr_lit.lit {
    //                     base_ident = Some(syn::parse_str(&litstr.value()).unwrap());
    //                 }
    //             }
    //         } else if key == "additions" {
    //             if let Expr::Lit(expr_lit) = &nv.value {
    //                 if let Lit::Str(litstr) = &expr_lit.lit {
    //                     additions_ident = Some(syn::parse_str(&litstr.value()).unwrap());
    //                 }
    //             }
    //         } else if key == "deletions" {
    //             if let Expr::Lit(expr_lit) = &nv.value {
    //                 if let Lit::Str(litstr) = &expr_lit.lit {
    //                     let deletions: Vec<String> = litstr
    //                         .value()
    //                         .split(',')
    //                         .map(|s| s.trim().to_string())
    //                         .collect();
    //                     deletions_ident = Some(deletions);
    //                 }
    //             }
    //         }
    //     }
    // }

    // println!("additions_ident={additions_ident:#?}");

    let base_expr = base.expect("Base mappings not provided");
    let additions_expr = additions.expect("Additions mappings not provided");
    let deletions_vec = deletions.expect("Deletions list not provided");
    eprintln!("base_expr={base_expr:#?}");
    eprintln!("additions_expr={additions_expr:#?}");

    let additions = quote! {
            #additions_expr
    };
    eprintln!("additions={additions:#?}");

    // Generate the final mappings from base, additions, and deletions
    let final_mappings = quote! {
        &[(i32, &str, &str)] = {
            let mut mappings = &#base_expr;
            // mappings.extend_from_slice(&#additions_expr);
            mappings
        }
    };

    eprintln!("final_mappings={final_mappings:#?}");

    // Return the struct along with the final mappings
    let output = quote! {
        #input_struct
        const FINAL_MAPPINGS: #final_mappings;
        const ADDITIONS: &[(i32, &str, &str)] = &#additions;
    };

    let token_stream = TokenStream::from(output);
    eprintln!("token_stream={:#?}", token_stream.to_string());
    token_stream
}
