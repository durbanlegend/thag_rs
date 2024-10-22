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
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes!
    let VecField { items } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Generate token stream for the items
    let item_tokens = items.iter().map(|(num, text)| {
        quote::quote! {
            println!("Number: {0}, Text: {1}", #num, #text);
        }
    });

    Ok(quote::quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            pub fn print_values(&self) {
                #( #item_tokens )*
            }
        }
    })
}
