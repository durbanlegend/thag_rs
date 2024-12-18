#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(my_desc))]
struct MyDescription {
    name: String,
    version: String,
}

pub fn my_derive(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes!
    let MyDescription { name, version } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote::quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            fn my_desc(&self) -> &'static str {
                concat!("Name: ", #name, ", Version: ", #version)
            }
        }
    })
}
