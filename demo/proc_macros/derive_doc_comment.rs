#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn derive_doc_comment_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = &input.ident;

    match input.data {
        Data::Enum(data_enum) => {
            // Handle enums - extract doc comments from variants
            let mut variant_idents = Vec::new();
            let mut variant_docs = Vec::new();

            for variant in &data_enum.variants {
                match parse_doc_comment(&variant.attrs) {
                    Some(doc) => {
                        variant_idents.push(&variant.ident);
                        variant_docs.push(doc);
                    }
                    None => {
                        return syn::Error::new_spanned(
                            &variant.ident,
                            "All enum variants must have doc comments when using DeriveDocComment",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }

            // Generate pattern matches based on variant types
            let variant_matches =
                data_enum
                    .variants
                    .iter()
                    .zip(variant_docs.iter())
                    .map(|(variant, doc)| {
                        let variant_name = &variant.ident;
                        match &variant.fields {
                            syn::Fields::Named(_) => quote! { Self::#variant_name { .. } => #doc },
                            syn::Fields::Unnamed(_) => quote! { Self::#variant_name(..) => #doc },
                            syn::Fields::Unit => quote! { Self::#variant_name => #doc },
                        }
                    });

            let expanded = quote! {
                impl #struct_name {
                    /// Returns the documentation comment for this enum variant.
                    pub fn doc_comment(&self) -> &'static str {
                        match self {
                            #( #variant_matches ),*
                        }
                    }

                    /// Returns all available documentation comments for this enum.
                    pub fn all_docs() -> &'static [(&'static str, &'static str)] {
                        &[
                            #( (stringify!(#variant_idents), #variant_docs) ),*
                        ]
                    }
                }
            };

            TokenStream::from(expanded)
        }
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(ref fields) => {
                    // Handle structs with named fields - extract field documentation
                    let mut field_names = Vec::new();
                    let mut field_docs = Vec::new();
                    let mut field_types = Vec::new();

                    for field in &fields.named {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;

                        if let Some(doc) = parse_doc_comment(&field.attrs) {
                            field_names.push(field_name);
                            field_docs.push(doc);
                            field_types.push(field_type);
                        } else {
                            // For structs, we'll include fields without docs with a default message
                            field_names.push(field_name);
                            field_docs.push("No documentation available".to_string());
                            field_types.push(field_type);
                        }
                    }

                    let struct_doc = parse_doc_comment(&input.attrs)
                        .unwrap_or_else(|| "No documentation available".to_string());

                    let expanded = quote! {
                        impl #struct_name {
                            /// Returns documentation for a specific field by name.
                            pub fn field_doc(field_name: &str) -> Option<&'static str> {
                                match field_name {
                                    #( stringify!(#field_names) => Some(#field_docs) ),*,
                                    _ => None,
                                }
                            }

                            /// Returns all field documentation as (name, type, doc) tuples.
                            pub fn all_field_docs() -> &'static [(&'static str, &'static str, &'static str)] {
                                &[
                                    #( (stringify!(#field_names), stringify!(#field_types), #field_docs) ),*
                                ]
                            }

                            /// Returns the struct's own documentation if available.
                            pub fn struct_doc() -> &'static str {
                                #struct_doc
                            }
                        }
                    };

                    TokenStream::from(expanded)
                }
                Fields::Unnamed(_) => {
                    // Handle tuple structs
                    let struct_doc = parse_doc_comment(&input.attrs)
                        .unwrap_or_else(|| "No documentation available".to_string());

                    let expanded = quote! {
                        impl #struct_name {
                            /// Returns the tuple struct's documentation.
                            pub fn struct_doc() -> &'static str {
                                #struct_doc
                            }
                        }
                    };

                    TokenStream::from(expanded)
                }
                Fields::Unit => {
                    // Handle unit structs
                    let struct_doc = parse_doc_comment(&input.attrs)
                        .unwrap_or_else(|| "No documentation available".to_string());

                    let expanded = quote! {
                        impl #struct_name {
                            /// Returns the unit struct's documentation.
                            pub fn struct_doc() -> &'static str {
                                #struct_doc
                            }
                        }
                    };

                    TokenStream::from(expanded)
                }
            }
        }
        Data::Union(_) => {
            syn::Error::new_spanned(struct_name, "DeriveDocComment does not support unions")
                .to_compile_error()
                .into()
        }
    }
}

/// Parses documentation comments from attributes.
///
/// Extracts the content of `#[doc = "..."]` attributes and combines multiple
/// doc comments into a single string with newlines.
pub(crate) fn parse_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_parts = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        if let Ok(meta) = attr.meta.require_name_value() {
            if let syn::Expr::Lit(expr_lit) = &meta.value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    doc_parts.push(lit_str.value().trim().to_string());
                }
            }
        }
    }

    if doc_parts.is_empty() {
        None
    } else {
        Some(doc_parts.join("\n"))
    }
}
