use proc_macro::TokenStream;
use quote::quote;

pub fn derive_doc_comment_impl(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, data, .. } = syn::parse_macro_input!(input);

    let (idents, docs) = match data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let mut idents = Vec::new();
            let mut docs = Vec::new();
            for var in &variants {
                match parse_enum_doc_comment(&var.attrs) {
                    Some(v) => docs.push(v),
                    None => panic!("doc comment is missing"),
                };

                idents.push(var.ident.clone());
            }

            (idents, docs)
        }
        _ => {
            panic!("only enums are supported")
        }
    };

    let output = quote! {
        impl #ident {
            fn doc_comment(&self) -> &'static str {
                match self {
                    #( #ident::#idents => #docs ),*
                }
            }
        }
    };

    output.into()
}

pub(crate) fn parse_enum_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        let meta = attr.meta.require_name_value().ok()?;
        if let syn::Expr::Lit(expr_lit) = &meta.value {
            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                return Some(lit_str.value().trim().to_string());
            }
        }
    }

    None
}
