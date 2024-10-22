#[allow(dead_code)]
// Deluxe struct for extracting attributes
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(deluxe))]
pub(crate) struct VecField {
    pub(crate) items: Vec<(i32, String)>, // We want this Vec<(i32, String)> in the struct
                                          // items: String,
}

pub(crate) fn deserialize_vec_derive_impl(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let input = syn::parse2::<syn::DeriveInput>(item)?;

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

    // Generate the code for the impl, using the mappings constant from the caller
    let ident = &input.ident;
    let output = quote::quote! {
        impl #ident {
            pub fn print_values(&self) {
                for (num, text) in #mappings_ident {
                    println!("Number: {}, Text: {}", num, text);
                }
            }
        }
    };
    Ok(output.into())
}
