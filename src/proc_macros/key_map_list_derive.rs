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

    // Extract the attributes!
    let KeyMappings { delete, add } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

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

    // Generate token stream for the deletions
    let delete_keys: Vec<_> = delete.iter().collect();
    // Generate token stream for the additions
    let add_tuples: Vec<_> = add.iter().collect();

    // Generate the code for the impl, using the mappings constant from the caller
    let ident = &input.ident;

    Ok(quote::quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            pub const fn adjust_mappings() -> &'static [(i32, &'static str, &'static str)] {
                // Base mappings from named constant (e.g., MAPPINGS)
                const BASE_MAPPINGS: &'static [(i32, &'static str, &'static str)] = #mappings_ident;

                // Deletions
                const DELETE_KEYS: &'static [&'static str] = &[#(#delete_keys),*];

                // Additions
                const ADD_MAPPINGS: &'static [(i32, &'static str, &'static str)] = &[
                    #(#add_tuples),*
                ];

                // Create the adjusted mappings
                const fn is_deleted(key: &str) -> bool {
                    let mut i = 0;
                    while i < DELETE_KEYS.len() {
                        if DELETE_KEYS[i] == key {
                            return true;
                        }
                        i += 1;
                    }
                    false
                }

                const fn adjusted_mappings() -> &'static [(i32, &'static str, &'static str)] {
                    let mut result = BASE_MAPPINGS;
                    let mut i = 0;
                    while i < BASE_MAPPINGS.len() {
                        if is_deleted(BASE_MAPPINGS[i].1) {
                            result = &result[1..]; // Remove the element
                        }
                        i += 1;
                    }
                    result
                }

                // Return final mappings (adjusted)
                adjusted_mappings()
            }
        }
    })
}
