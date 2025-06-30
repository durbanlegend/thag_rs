#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive_constructor_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = &input.ident;

    // Extract fields from the struct
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "DeriveConstructor only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "DeriveConstructor can only be applied to structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate parameter list for the new() function
    let params = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! { #field_name: #field_type }
    });

    // Generate field assignments for the constructor
    let field_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name }
    });

    let expanded = quote! {
        impl #struct_name {
            pub fn new(#(#params),*) -> #struct_name {
                Self {
                    #(#field_assignments),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
