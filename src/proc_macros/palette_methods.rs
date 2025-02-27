#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn palette_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => panic!("PaletteMethods only works with named fields"),
        },
        _ => panic!("PaletteMethods only works with structs"),
    };

    let validation_calls = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            validate_style(&self.#field_name, min_support)?;
        }
    });

    let conversion_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            #field_name: Style::from_config(&config.#field_name)?
        }
    });

    // New: Generate style references for iterator
    let style_refs = fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! {
            &mut self.#field_name
        }
    });

    let output = quote! {
        impl Palette {
            /// Validates all styles in the palette against the minimum color support level.
            ///
            /// # Arguments
            /// * `min_support` - The minimum color support level required by the theme
            ///
            /// # Returns
            /// * `Ok(())` if all styles are valid for the given support level
            /// * `Err(ThemeError)` if any style requires higher color support than available
            pub fn validate_styles(&self, min_support: ColorSupport) -> ThagResult<()> {
                #(#validation_calls)*
                Ok(())
            }

            /// Creates a new Palette from a PaletteConfig
            ///
            /// Converts all StyleConfig entries to their corresponding Style values
            ///
            /// # Arguments
            /// * `config` - The PaletteConfig containing the style definitions
            ///
            /// # Returns
            /// * `Ok(Palette)` if all conversions succeed
            /// * `Err(ThemeError)` if any conversion fails
            pub fn from_config(config: &PaletteConfig) -> ThagResult<Self> {
                Ok(Self {
                    #(#conversion_fields,)*
                })
            }

            // New method to get mutable iterator over all styles
             pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Style> {
                 vec![
                     #(#style_refs,)*
                 ].into_iter()
             }
        }
    };

    output.into()
}
