// Deluxe struct for extracting attributes
#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(deluxe))]
pub(crate) struct KeyMappings {
    pub(crate) delete: Vec<String>,
    pub(crate) add: Vec<(i32, String, String)>,
}

pub(crate) fn key_map_list_derive_impl(
    item: proc_macro2::TokenStream,
) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut input = syn::parse2::<syn::DeriveInput>(item)?;

    // Extract the attributes!
    let KeyMappings { delete, add } = deluxe::extract_attributes(&mut input)?;

    // Now get some info to generate an associated function...
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Generate token stream for the deletions
    let delete_tokens = delete.iter().map(|key| {
        quote::quote! {
            println!("Key: {}", #key);
        }
    });
    // Generate token stream for the additions
    let add_tokens = add.iter().map(|(seq, key, desc)| {
        quote::quote! {
            println!("Number: {0}, Key: {1}, Desc: {2}", #seq, #key, #desc);
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
    Ok(quote::quote! {
        impl #impl_generics #ident #type_generics #where_clause {
            pub fn adjust_mappings(&self) -> &'static[(i32, String, String)] {
                println!("Base mappings from named constant:");
                for (seq, key, desc) in #mappings_ident {
                    println!("Seq: {}, key: {}, desc: {}", seq, key, desc);
                }
                println!("Deletions:");
                #( #delete_tokens )*
                println!("Additions:");
                #( #add_tokens )*

                static ADJUSTED_MAPPINGS: OnceLock<&'static[(i32, String, String)]> = OnceLock::new();
                let adjusted_mappings = ADJUSTED_MAPPINGS.get_or_init(|| {
                    #mappings_ident
                        .iter()
                        .filter(|&row| !#delete.contains(&row.keys))
                        .chain(#add.iter())
                        .cloned()
                        .collect()
                });
                let range = &'static adjusted_mappings[..];
                range
            }
        }
    })
}
