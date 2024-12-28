/// Published example from crate `expander`
use expander::{Edition, Expander};

// or any other macro type
pub fn baz2(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let modified = quote::quote! {
        #[derive(Debug, Clone, Copy)]
        #input
    };

    let expanded = Expander::new("baz")
        .add_comment("This is generated code!".to_owned())
        .fmt(Edition::_2021)
        .verbose(true)
        // common way of gating this, by making it part of the default feature set
        // .dry(cfg!(feature = "no-file-expansion"))
        .dry(false)
        .write_to_out_dir(modified.clone())
        .unwrap_or_else(|e| {
            eprintln!("Failed to write to file: {:?}", e);
            modified
        });
    expanded
}
