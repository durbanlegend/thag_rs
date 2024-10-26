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
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes!
    let VecField { items } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    eprintln!("impl_generics={impl_generics:?}\ntype_generics={type_generics:?}\nwhere_clause={where_clause:?}");
    // Generate token stream for the items
    let item_tokens = items.iter().map(|(num, text)| {
        quote::quote! {
            println!("Number: {0}, Text: {1}", #num, #text);
        }
    });

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
                println!("Values from named constant:");
                for (num, text) in #mappings_ident {
                    println!("Number: {}, Text: {}", num, text);
                }
                println!("Values from attribute literals:");
                #( #item_tokens )*
            }
        }
    };
    Ok(output.into())
}
