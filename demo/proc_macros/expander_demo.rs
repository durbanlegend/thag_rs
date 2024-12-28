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
        // Formatting seems to be getting OS error 33 sometimes on Windows:
        // "error: couldn't read \\?\C:\Users\runneradmin\AppData\Local\Temp\thag_rs\proc_macro_expander_demo\target\debug\build\expander-29b18d460c115a5a\out\baz_0-3535fb96b397.rs: The process cannot access the file because another process has locked a portion of the file. (os error 33)"
        // .fmt(Edition::_2021)
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
