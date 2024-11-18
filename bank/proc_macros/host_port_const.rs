#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, Meta};

pub fn host_port_const_impl(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let var_name = parse_macro_input!(input as DeriveInput);
    let input = var_name;
    let struct_name = input.ident;

    // Collect generated constants
    let mut consts = Vec::new();

    if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(fields_named) = data_struct.fields {
            for field in fields_named.named {
                for attr in &field.attrs {
                    // Check if the attribute is `const_value`
                    if let Meta::NameValue(meta_name_value) = &attr.meta {
                        let segments = &meta_name_value.path.segments;
                        if !segments.is_empty() && segments[0].ident == "const_value" {
                            // Parse the attribute metadata
                            match &meta_name_value.value {
                                Expr::Lit(expr_lit) => {
                                    if let Lit::Str(lit_str) = &expr_lit.lit {
                                        let field_name = format_ident!(
                                            "{}",
                                            field.ident.clone().unwrap().to_string().to_uppercase()
                                        );
                                        println!("field_name={field_name}");
                                        let value = lit_str.value(); // Extract the string value

                                        // Push the generated constant
                                        consts.push(quote! {
                                            pub const #field_name: &'static str = #value;
                                        });
                                    }
                                }
                                _ => (),
                            }
                        };
                    }
                }
            }
        }
    }

    // Generate the implementation with the constants
    let expanded = quote! {
        impl #struct_name {
            #(#consts)*
        }
    };

    TokenStream::from(expanded)
}
