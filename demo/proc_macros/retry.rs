#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Lit, LitInt};

pub fn retry_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Parse the retry count - simplified approach
    let retry_count = if attr.is_empty() {
        3 // Default retry count
    } else {
        // Try to parse as a simple integer first
        if let Ok(lit_int) = syn::parse::<LitInt>(attr.clone()) {
            lit_int.base10_parse().unwrap_or(3)
        } else {
            // Try to parse attribute syntax like "times = 5"
            let attr_str = attr.to_string();
            if let Some(equals_pos) = attr_str.find('=') {
                let value_part = attr_str[equals_pos + 1..].trim();
                value_part.parse().unwrap_or(3)
            } else {
                3
            }
        }
    };

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    let fn_name_str = fn_name.to_string();

    let result = quote! {
        #(#attrs)*
        #vis fn #fn_name #generics(#inputs) #output #where_clause {
            let mut last_error: Option<String> = None;

            for attempt in 1..=#retry_count {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    #block
                }));

                match result {
                    Ok(value) => {
                        if attempt > 1 {
                            println!("🔄 Function '{}' succeeded on attempt {}/{}", #fn_name_str, attempt, #retry_count);
                        }
                        return value;
                    }
                    Err(panic_info) => {
                        if attempt < #retry_count {
                            println!("⚠️  Function '{}' failed on attempt {}/{}, retrying...", #fn_name_str, attempt, #retry_count);

                            // Small delay between retries
                            std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
                        } else {
                            println!("❌ Function '{}' failed after {} attempts", #fn_name_str, #retry_count);
                        }

                        // Convert panic to error for the last attempt
                        if attempt == #retry_count {
                            std::panic::resume_unwind(panic_info);
                        }
                    }
                }
            }

            unreachable!("Retry loop should always return or panic")
        }
    };

    result.into()
}
