#![allow(unused_variables)]
// Deluxe struct for extracting attributes
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(deluxe))]
pub(crate) struct KeyMappings {
    pub(crate) delete: Vec<String>,
    pub(crate) add: Vec<(i32, String, String)>,
}

pub(crate) fn key_map_list_derive_impl(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes
    let KeyMappings { delete, add } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Generate token stream for the deletions
    let delete_tokens = delete.iter().map(|key| {
        quote::quote! { #key, }
    });

    // eprintln!("delete_tokens={delete_tokens:#?}");

    // Generate token stream for the additions
    let add_tokens = add.iter().map(|(seq, key, desc)| {
        quote::quote! {
            ( #seq, #key.to_string(), #desc.to_string() ),
        }
    });

    // Extract the 'use_mappings' attribute from the struct
    let mappings_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("use_mappings"));

    // Assume the attribute is found and contains the MAPPINGS identifier
    let mappings_ident = if let Some(attr) = mappings_attr {
        // Parse the attribute as an expression (since it can now be any expression)
        attr.parse_args::<syn::Expr>().ok()
    } else {
        None
    }
    .expect("Must provide a 'use_mappings' attribute");

    // Generate the code for the impl, using the mappings constant from the caller
    let ident = &input.ident;
    Ok(quote::quote! {
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
    })
}
