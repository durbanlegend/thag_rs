#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[allow(clippy::too_many_lines)]
pub fn derive_display_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = &input.ident;

    // Handle both structs and enums
    let display_impl = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => {
                // For structs with named fields, create a formatted display
                let fields = &fields_named.named;

                // Generate field display parts (using field_displays_with_separators instead)

                let field_count = fields.len();
                let field_displays_with_separators = fields.iter().enumerate().map(|(i, field)| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();

                    if i == field_count - 1 {
                        // Last field, no comma
                        quote! {
                            write!(f, "{}: {}", #field_name_str, self.#field_name)?;
                        }
                    } else {
                        // Not last field, add comma and space
                        quote! {
                            write!(f, "{}: {}, ", #field_name_str, self.#field_name)?;
                        }
                    }
                });

                quote! {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{} {{ ", stringify!(#struct_name))?;
                        #(#field_displays_with_separators)*
                        write!(f, " }}")
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                // For tuple structs, display fields by index
                let field_displays = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                    let index = syn::Index::from(i);
                    if i == fields_unnamed.unnamed.len() - 1 {
                        // Last field
                        quote! {
                            write!(f, "{}", self.#index)?;
                        }
                    } else {
                        // Not last field
                        quote! {
                            write!(f, "{}, ", self.#index)?;
                        }
                    }
                });

                quote! {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}(", stringify!(#struct_name))?;
                        #(#field_displays)*
                        write!(f, ")")
                    }
                }
            }
            Fields::Unit => {
                // Unit struct
                quote! {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", stringify!(#struct_name))
                    }
                }
            }
        },
        Data::Enum(data_enum) => {
            // For enums, display variant name and any fields
            let variant_displays = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(fields) => {
                        // Enum variant with named fields
                        let field_patterns = fields.named.iter().map(|field| {
                            let field_name = field.ident.as_ref().unwrap();
                            quote! { #field_name }
                        });

                        let field_displays = fields.named.iter().enumerate().map(|(i, field)| {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_name_str = field_name.to_string();

                            if i == fields.named.len() - 1 {
                                quote! {
                                    write!(f, "{}: {}", #field_name_str, #field_name)?;
                                }
                            } else {
                                quote! {
                                    write!(f, "{}: {}, ", #field_name_str, #field_name)?;
                                }
                            }
                        });

                        quote! {
                            Self::#variant_name { #(#field_patterns),* } => {
                                write!(f, "{}::{} {{ ", stringify!(#struct_name), stringify!(#variant_name))?;
                                #(#field_displays)*
                                write!(f, " }}")
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        // Enum variant with unnamed fields (tuple-like)
                        let field_patterns = (0..fields.unnamed.len()).map(|i| {
                            quote::format_ident!("field_{}", i)
                        });

                        let field_displays = field_patterns.clone().enumerate().map(|(i, field_name)| {
                            if i == fields.unnamed.len() - 1 {
                                quote! {
                                    write!(f, "{}", #field_name)?;
                                }
                            } else {
                                quote! {
                                    write!(f, "{}, ", #field_name)?;
                                }
                            }
                        });

                        quote! {
                            Self::#variant_name(#(#field_patterns),*) => {
                                write!(f, "{}::{}(", stringify!(#struct_name), stringify!(#variant_name))?;
                                #(#field_displays)*
                                write!(f, ")")
                            }
                        }
                    }
                    Fields::Unit => {
                        // Unit variant
                        quote! {
                            Self::#variant_name => {
                                write!(f, "{}::{}", stringify!(#struct_name), stringify!(#variant_name))
                            }
                        }
                    }
                }
            });

            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#variant_displays,)*
                    }
                }
            }
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(struct_name, "DeriveDisplay does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl std::fmt::Display for #struct_name {
            #display_impl
        }
    };

    TokenStream::from(expanded)
}
