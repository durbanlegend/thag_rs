#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn enable_profiling_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Runtime check for feature flag to handle when the proc macro
    // is compiled with the feature but used without it
    if cfg!(not(feature = "profiling")) {
        // No wrapper, return original function
        return item;
    }

    // let args = parse_macro_input!(attr as ProfileArgs);
    let input = parse_macro_input!(item as ItemFn);

    // Check if the function is async
    let is_async = input.sig.asyncness.is_some();

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    for attr in attrs {
        assert_ne!(
            quote!(#attr).to_string().as_str(),
            "#[tokio :: main]",
            "#[tokio::main] if present must appear before #[enable_profiling] for correct expansion."
        );
    }

    // let maybe_fn_name = format!(r#"Some("{fn_name}")"#);
    let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    let result = if is_async {
        // Handle async function
        quote! {
            #(#attrs)*
            #vis async fn #fn_name #generics(#inputs) #output #where_clause {

                // Initialize profiling
                ::thag_profiler::init_profiling(module_path!());

                // Create a profile that covers everything, including tokio setup
                // We pass None for the name as we rely on the backtrace to identify the function
                let profile = ::thag_profiler::Profile::new(
                    None,
                    Some(#fn_name_str),
                    ::thag_profiler::profiling::get_global_profile_type(),
                    false,
                    false,
                );

                // For async functions, we need to use an async block
                let result = async {
                    #block
                }.await;

                // Drop the profile explicitly at the end
                drop(profile);

                // Finalize profiling
                ::thag_profiler::finalize_profiling();
                result
            }
        }
    } else {
        // Handle non-async function (existing implementation)
        quote! {
            #(#attrs)*
            #vis fn #fn_name #generics(#inputs) #output #where_clause {

                // Initialize profiling
                ::thag_profiler::init_profiling(module_path!());

                // Create a profile that covers everything, including tokio setup
                // We pass None for the name as we rely on the backtrace to identify the function
                let profile = ::thag_profiler::Profile::new(
                    None,
                    Some(#fn_name_str),
                    ::thag_profiler::profiling::get_global_profile_type(),
                    false,
                    false,
                );

                let result = (|| {
                    #block
                })();

                // Drop the profile explicitly at the end
                drop(profile);

                // Finalize profiling
                ::thag_profiler::finalize_profiling();
                result
            }
        }
    };

    result.into()
}
