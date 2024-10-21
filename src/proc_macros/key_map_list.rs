#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(my_desc))]
struct KeyMapList {
    // base: Vec<(syn::LitInt, syn::LitStr, syn::LitStr)>,
    remove: Vec<String>,
    // add: Vec<(syn::LitInt, syn::LitStr, syn::LitStr)>,
}

pub fn derive_key_map_list_impl(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes!
    let KeyMapList {
        // base: _,
        remove,
        // add: _,
    } = deluxe::extract_attributes(&mut input)?;

    eprintln!("input={input:?}");
    // Now get some info to generate an associated function...
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote::quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            fn my_desc(&self) -> Vec<String> {
                // concat!("Base: ", vec![#(#base),*], ", remove: ", vec![#(#remove),*], ", add: ", vec![#(#add),*],)
                vec![#(#remove),*]
            }
        }
    })
}
