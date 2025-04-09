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
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // Empty input means use default
        if input.is_empty() {
            return Ok(Self {
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

        Ok(Self { mode })
    }
}

#[allow(clippy::too_many_lines)]
pub fn enable_profiling_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(cfg!(feature = "time_profiling"));

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

    let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    let profile_new = quote! {
        ::thag_profiler::Profile::new(None, Some(#fn_name_str), ::thag_profiler::get_global_profile_type(), #is_async, false)
    };

    #[cfg(not(feature = "full_profiling"))]
    let profile_drop = quote! {
        drop(profile);
    };

    #[cfg(feature = "full_profiling")]
    let profile_drop = quote! {
        with_allocator(Allocator::System, || {
            drop(profile);
        });
    };

    #[cfg(not(feature = "full_profiling"))]
    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                use ::thag_profiler::{finalize_profiling, init_profiling};

                let should_profile = std::env::var("THAG_PROFILE").ok().is_some();
                eprintln!("should_profile={should_profile}");

                if should_profile {
                    // Initialize profiling
                    init_profiling(module_path!());
                }

                let maybe_profile = if should_profile {
                    #profile_new
                } else {
                    None
                };
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Initialize profiling
                init_profiling(module_path!());

                let profile = #profile_new;
            }
        }
        ProfilingMode::Disabled => {
            quote! {}
        }
    };

    #[cfg(feature = "full_profiling")]
    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                use ::thag_profiler::{finalize_profiling, init_profiling, with_allocator, Allocator};

                let (should_profile, maybe_profile) = with_allocator(Allocator::System, || {
                    let should_profile = std::env::var("THAG_PROFILE").ok().is_some();
                    eprintln!("should_profile={should_profile}");

                    if should_profile {
                        // Initialize profiling
                        ::thag_profiler::init_profiling(module_path!());
                    }

                    let maybe_profile = if should_profile {
                        #profile_new
                    } else {
                        None
                    };
                    (should_profile, maybe_profile)
                });
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Initialize profiling
                init_profiling(module_path!());

                let profile = #profile_new;
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
                        #profile_drop
                    }

                    // Finalize profiling
                    finalize_profiling();
                }
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Drop the profile explicitly at the end
                #profile_drop

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
