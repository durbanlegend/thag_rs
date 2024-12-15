use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data};

/// Convert a struct to a `HashMap`.
///
/// # Panics
///
/// Panics if no ident found for field.
pub fn into_hash_map_impl(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_identifier = &input.ident;

    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            let field_identifiers = fields.iter().map(|item| item.ident.as_ref().unwrap()).collect::<Vec<_>>();

            quote! {
                impl From<#struct_identifier> for std::collections::HashMap<String, String> {
                    fn from(value: #struct_identifier) -> Self {
                        let mut hash_map = std::collections::HashMap::<String, String>::new();

                        #(
                            hash_map.insert(stringify!(#field_identifiers).to_string(), String::from(value.#field_identifiers));
                        )*

                        hash_map
                    }
                }
            }
        }
        _ => unimplemented!()
    }.into()
}
