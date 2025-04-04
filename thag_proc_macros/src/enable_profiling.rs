#![allow(clippy::module_name_repetitions)]
use std::result::Result;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, ItemFn,
};

#[derive(Default)]
pub enum ProfilingMode {
    Runtime, // Check environment variable at runtime
    #[default]
    Enabled, // Always enabled
    Disabled, // Always disabled
}

/// Configuration for `enable_profiling` attribute macro
#[derive(Default)]
struct ProfilingArgs {
    mode: ProfilingMode,
}

impl Parse for ProfilingArgs {
    fn parse(input: ParseStream) -> Result<ProfilingArgs, syn::Error> {
        // Empty input means use default
        if input.is_empty() {
            return Ok(ProfilingArgs {
                mode: ProfilingMode::Enabled,
            });
        }

        // Parse the mode identifier
        let mode_ident: Ident = input.parse()?;
        let mode_str = mode_ident.to_string();

        let mode = match mode_str.as_str() {
            "runtime" => ProfilingMode::Runtime,
            "yes" => ProfilingMode::Enabled,
            "no" => ProfilingMode::Disabled,
            _ => {
                return Err(syn::Error::new(
                    mode_ident.span(),
                    "Expected 'runtime', 'yes', or 'no'",
                ));
            }
        };

        Ok(ProfilingArgs { mode })
    }
}

pub fn enable_profiling_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Runtime check for feature flag to handle when the proc macro
    // is compiled with the feature but used without it
    if cfg!(not(feature = "profiling")) {
        // No wrapper, return original function
        return item;
    }

    let args = parse_macro_input!(attr as ProfilingArgs);
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

    let new_profile = quote! {
        // Create a profile that covers everything, including tokio setup
        // We pass None for the name as we rely on the backtrace to identify the function
        ::thag_profiler::Profile::new(
            None,
            Some(#fn_name_str),
            ::thag_profiler::profiling::get_global_profile_type(),
            false,
            false,
        )
    };

    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                let should_profile = std::env::var("ENABLE_PROFILING")
                    .map(|val| val == "1" || val.to_lowercase() == "true")
                    .unwrap_or(false);
                eprintln!("should_profile={should_profile}");

                if should_profile {
                    // Initialize profiling
                    ::thag_profiler::init_profiling(module_path!());
                }

                let maybe_profile = if should_profile {
                    Some(#new_profile)
                } else {
                    None
                };
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Initialize profiling
                ::thag_profiler::init_profiling(module_path!());

                let profile = #new_profile;
            }
        }
        ProfilingMode::Disabled => {
            quote! {}
        }
    };

    let profile_finalize = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                if should_profile {
                    // Drop the profile explicitly at the end
                    if let Some(profile) = maybe_profile {
                        drop(profile);
                    }

                    // Finalize profiling
                    ::thag_profiler::finalize_profiling();
                }
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Drop the profile explicitly at the end
                drop(profile);

                // Finalize profiling
                ::thag_profiler::finalize_profiling();
            }
        }
        ProfilingMode::Disabled => {
            quote! {}
        }
    };

    let async_token = if is_async { quote!(async) } else { quote!() };

    let wrapped_block = if is_async {
        quote! {
            // For async functions, we need to use an async block
            let result = async {
                #block
            }.await;
        }
    } else {
        quote! {
            let result = (|| {
                #block
            })();
        }
    };

    let result = quote! {
        #(#attrs)*
        #vis #async_token fn #fn_name #generics(#inputs) #output #where_clause {

            #profile_init

            #wrapped_block

            #profile_finalize

            result
        }
    };

    result.into()
}
