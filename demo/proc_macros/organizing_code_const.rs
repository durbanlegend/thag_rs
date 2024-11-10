#![allow(dead_code, unused_variables, clippy::redundant_pub_crate)]
/// Experimental - work in progress
use expander::{Edition, Expander};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

// Deluxe struct for extracting attributes
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(adjust))]
pub(crate) struct KeyMappings {
    pub(crate) delete: Vec<String>,
    pub(crate) add: Vec<(i32, String, String)>,
}

pub(crate) fn organizing_code_const_impl(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes
    let KeyMappings { delete, add } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Generate token stream for the deletions
    let delete_tokens = delete.iter().map(|key| {
        quote! { #key, }
    });

    // eprintln!("delete_tokens={delete_tokens:#?}");

    // Generate token stream for the additions
    let add_tokens = add.iter().map(|(seq, key, desc)| {
        quote! {
            ( #seq, #key.to_string(), #desc.to_string() ),
        }
    });

    // Extract the 'use_mappings' attribute from the struct
    let mappings_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("use_mappings"));
    eprintln!("mappings_attr={mappings_attr:#?}");

    let mappings: syn::Expr = parse_quote!(println!(#mappings_attr));

    eprintln!("mappings={mappings:#?}");

    // let output = quote!(#mappings_attr);

    // Assume the attribute is found and contains the MAPPINGS identifier
    let mappings_ident = mappings_attr
        .map_or_else(|| None, |attr| attr.parse_args::<syn::Expr>().ok())
        .expect("Must provide a 'use_mappings' attribute");

    // Generate the code for the impl, using the mappings constant from the caller
    let ident = &input.ident;
    let output = quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            pub fn adjust_mappings(&self) -> Vec<(i32, String, String)> {
                // eprintln!("Base mappings from named constant:");
                // for (seq, key, desc) in #mappings_ident {
                //     eprintln!("Seq: {}, key: {}, desc: {}", seq, key, desc);
                // }
                static ADJUSTED_MAPPINGS: std::sync::OnceLock<Vec<(i32, String, String)>> =
                    std::sync::OnceLock::new();
                let adjusted_mappings = ADJUSTED_MAPPINGS.get_or_init(move || {
                    const BASE_MAPPINGS: &[(i32, &'static str, &'static str)] = &MAPPINGS;
                    let mut work_vec: Vec<(i32, String, String)> = vec![];
                    let deletions = vec![ #( #delete_tokens )* ];
                    // eprintln!("deletions={deletions:#?}");
                    let additions = vec![ #( #add_tokens )* ];
                    for mapping in BASE_MAPPINGS {
                        // eprintln!("mapping.1={:#?}", mapping.1);
                        // if !(mapping.1 == "I" || mapping.1 == "u") {
                        if !(deletions.contains(&mapping.1)) {
                            work_vec.push((mapping.0, String::from(mapping.1), String::from(mapping.2)));
                        }
                    }
                    for row in additions {
                        // eprintln!("row={row:#?}");
                        work_vec.push(row);
                    }
                    work_vec
                });
                // eprintln!("adjusted_mappings={adjusted_mappings:#?}");
                adjusted_mappings.clone()
            }
        }
    };

    let expanded = Expander::new("DeriveConst")
        .add_comment("This is generated code!".to_owned())
        .fmt(Edition::_2021)
        .verbose(true)
        .dry(false)
        .write_to_out_dir(output.clone())
        .unwrap_or_else(|e| {
            eprintln!("Failed to write to file: {:?}", e);
            output
        });
    Ok(expanded)
}

// pub fn organizing_code_const_impl(input: TokenStream) -> TokenStream {
//     let progress = progress_message("Thinking about the answer".to_string());
//     let answer = answer(input);

//     // println!("answer={answer:#?}");

//     quote!(
//         #progress;
//         #answer;
//     )
// }

// fn progress_message(msg: String) -> ExprMacro {
//     parse_quote!(println!(#msg))
// }

// fn answer(result: TokenStream) -> ExprMacro {
//     parse_quote!(println!("Answer: {}", #result))
// }
