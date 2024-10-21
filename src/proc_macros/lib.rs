mod custom_model;
mod into_string_hash_map;
mod key_map_list;
mod my_description;
mod tui_keys;

use crate::custom_model::derive_custom_model_impl;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::key_map_list::derive_key_map_list_impl;
use crate::my_description::my_derive;
use crate::tui_keys::key_impl;
use proc_macro::TokenStream;

// Not public API. This is internal and to be used only by `key!`.
#[doc(hidden)]
#[proc_macro]
pub fn key(input: TokenStream) -> TokenStream {
    key_impl(input)
}

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(item: TokenStream) -> TokenStream {
    derive_custom_model_impl(item)
}

#[proc_macro_derive(IntoStringHashMap)]
pub fn into_hash_map(item: TokenStream) -> TokenStream {
    into_hash_map_impl(item)
}

#[proc_macro_derive(DeriveKeyMapList, attributes(my_desc))]
pub fn derive_key_map_list(item: TokenStream) -> proc_macro::TokenStream {
    derive_key_map_list_impl(item.into()).unwrap().into()
}

#[proc_macro_derive(MyDescription, attributes(my_desc))]
pub fn derive_my_description(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    my_derive(item.into()).unwrap().into()
}

// // Define the custom derive macro using `deluxe`
// #[proc_macro_derive(DeserializeVec, attributes(deluxe))]
// pub fn deserialize_vec_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     deserialize_vec_derive_impl(input.into()).unwrap().into()
// }

// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(deluxe))]
// // #[derive(deluxe::ParseMetaItem)]
// struct VecField {
//     items: Vec<(i32, String)>,
// }

// fn deserialize_vec_derive_impl(
//     input: proc_macro2::TokenStream,
// ) -> deluxe::Result<proc_macro2::TokenStream> {
//     // let mut input = parse_macro_input!(input as DeriveInput);
//     let mut input = syn::parse2::<syn::DeriveInput>(input)?;

//     // Parse struct attributes using `deluxe`
//     let VecField { items } = deluxe::extract_attributes(&mut input.attrs)?;

//     // Get the struct name
//     let name = input.ident;

//     // Implement a custom method on the struct to handle the Vec<(i32, String)>
//     let expanded = quote::quote! {
//         impl #name {
//             pub fn print_values(&self) {
//                 println!("In print_values()!");
//                 for (num, text) in &self.items {
//                     println!("Number: {}, Text: {}", num, text);
//                 }
//             }
//         }
//     };

//     Ok(expanded)
// }

// Define the custom derive macro using `deluxe`
#[proc_macro_derive(DeserializeVec, attributes(deluxe))]
pub fn deserialize_vec_derive(input: TokenStream) -> TokenStream {
    deserialize_vec_derive_impl(input.into()).unwrap().into()
}

// Deluxe struct for extracting attributes
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(deluxe))]
struct VecField {
    items: Vec<(i32, String)>, // We want this Vec<(i32, String)> in the struct
                               // items: String,
}

fn deserialize_vec_derive_impl(
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
