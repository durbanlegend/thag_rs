/// Published example from crate `expander`
use expander::{Edition, Expander};
use std::{thread, time::Duration};

pub fn baz2(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let modified = quote::quote! {
        #[derive(Debug, Clone, Copy)]
        #input
    };

    // Try multiple times with increasing delays
    for attempt in 0..3 {
        let result = Expander::new(&format!("baz_{}", attempt)) // Make each attempt use a unique name
            .add_comment("This is generated code!".to_owned())
            .fmt(Edition::_2021)
            .verbose(true)
            .dry(false)
            .write_to_out_dir(modified.clone());

        match result {
            Ok(expanded) => return expanded,
            Err(e) => {
                eprintln!("Attempt {} failed: {:?}", attempt, e);
                if attempt < 2 {
                    thread::sleep(Duration::from_millis(100 * (attempt + 1) as u64));
                }
            }
        }
    }

    // Fall back to the modified tokens if all attempts fail
    eprintln!("All attempts to write to file failed, falling back to direct expansion");
    modified
}
