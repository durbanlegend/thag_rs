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

    // Filter out deleted mappings from MAPPINGS
    let delete_tokens = delete.iter().map(|key| {
        let key = key.as_str();
        quote::quote! { #key }
    });

    // Generate token stream for the additions
    let add_tokens = add.iter().map(|(seq, key, desc)| {
        let key = key.as_str();
        let desc = desc.as_str();
        quote::quote! {
            ( #seq, #key, #desc ),
        }
    });

    // Assume the attribute is found and contains the MAPPINGS identifier
    let mappings_ident = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("use_mappings"));

    // Generate the code for the impl, filtering out deleted items and adding new ones
    let ident = &input.ident;
    Ok(quote::quote! {
        impl #ident {
            pub const fn adjust_mappings(&self) -> &'static [(i32, &'static str, &'static str)] {
                const BASE_MAPPINGS: &[(i32, &'static str, &'static str)] = &MAPPINGS;
                const FILTERED_MAPPINGS: Vec<(i32, &'static str, &'static str)> = {
                    let mut result = Vec::new();
                    for mapping in BASE_MAPPINGS.iter() {
                        if !(#(mapping.1 == #delete_tokens) || *) {
                            result.push(*mapping);
                        }
                    }
                    result
                };
                &[
                    #( #add_tokens )*,
                    &FILTERED_MAPPINGS
                ]
            }
        }
    })
}
