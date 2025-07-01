#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

pub fn derive_builder_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let struct_name = &input.ident;

    // Extract fields from the struct
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "DeriveBuilder only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "DeriveBuilder can only be applied to structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate builder struct name
    let builder_name = quote::format_ident!("{}Builder", struct_name);

    // Generate builder fields (all wrapped in Option)
    let builder_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        quote! {
            #field_name: Option<#field_type>
        }
    });

    // Generate builder setter methods
    let setter_methods = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Create method documentation
        let doc = format!("Sets the `{}` field.", field_name);

        quote! {
            #[doc = #doc]
            pub fn #field_name(mut self, #field_name: #field_type) -> Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    // Generate field assignments for build method with error handling
    let field_assignments = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        quote! {
            #field_name: self.#field_name.ok_or_else(||
                format!("Field '{}' is required but was not set", #field_name_str)
            )?
        }
    });

    // Generate new method for builder (initializes all fields to None)
    let builder_new_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            #field_name: None
        }
    });

    let expanded = quote! {
        /// Builder struct for constructing instances step by step.
        #[derive(Debug, Clone)]
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            /// Creates a new builder instance with all fields unset.
            pub fn new() -> Self {
                Self {
                    #(#builder_new_fields,)*
                }
            }

            #(#setter_methods)*

            /// Builds the final instance, returning an error if any required fields are missing.
            ///
            /// # Errors
            /// Returns an error message if any field has not been set.
            pub fn build(self) -> Result<#struct_name, String> {
                Ok(#struct_name {
                    #(#field_assignments,)*
                })
            }
        }

        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #struct_name {
            /// Creates a new builder for this struct.
            ///
            /// # Examples
            /// ```
            /// let instance = MyStruct::builder()
            ///     .field1(value1)
            ///     .field2(value2)
            ///     .build()?;
            /// ```
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    };

    TokenStream::from(expanded)
}
