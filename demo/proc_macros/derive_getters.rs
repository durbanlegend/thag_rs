#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn derive_getters_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = &input.ident;

    // Extract fields from the struct
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "DeriveGetters only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "DeriveGetters can only be applied to structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate getter methods for each field
    let getters = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Create getter method name
        let getter_name = field_name;

        // Check if the field type is a reference or needs special handling
        let return_type = match field_type {
            Type::Reference(_) => quote! { #field_type },
            _ => {
                // For owned types, return a reference to avoid moving
                quote! { &#field_type }
            }
        };

        // Generate getter method with documentation
        quote! {
            /// Gets a reference to the `#field_name` field.
            pub fn #getter_name(&self) -> #return_type {
                &self.#field_name
            }
        }
    });

    let expanded = quote! {
        impl #struct_name {
            #(#getters)*
        }
    };

    TokenStream::from(expanded)
}
