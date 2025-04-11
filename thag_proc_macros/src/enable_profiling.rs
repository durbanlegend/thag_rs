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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ProfileType {
    Time, // Wall clock/elapsed time
    Memory,
    #[default]
    Both,
}

/// Configuration for `enable_profiling` attribute macro
#[derive(Default)]
struct ProfilingArgs {
    mode: ProfilingMode,
    profile_type: Option<ProfileType>,
}

impl Parse for ProfilingArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // Empty input means use default
        #[cfg(not(feature = "full_profiling"))]
        if input.is_empty() {
            return Ok(Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Time),
            });
        }

        #[cfg(feature = "full_profiling")]
        if input.is_empty() {
            return Ok(Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Both),
            });
        }

        // Parse the mode identifier
        let mode_ident: Ident = input.parse()?;
        let mode_str = mode_ident.to_string();

        Ok(match mode_str.as_str() {
            "no" => Self {
                mode: ProfilingMode::Disabled,
                profile_type: None,
            },
            "runtime" => Self {
                mode: ProfilingMode::Runtime,
                profile_type: None,
            },
            "both" => Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Both),
            },
            "yes" => Self {
                mode: ProfilingMode::Enabled,
                #[cfg(feature = "full_profiling")]
                profile_type: Some(ProfileType::Both),
                #[cfg(not(feature = "full_profiling"))]
                profile_type: Some(ProfileType::Time),
            },
            "memory" => Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Memory),
            },
            "time" => Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Time),
            },
            _ => {
                return Err(syn::Error::new(
                    mode_ident.span(),
                    "Expected 'memory', 'time', 'both', 'runtime', 'yes', or 'no'",
                ));
            }
        })
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
                use ::thag_profiler::{finalize_profiling, init_profiling, PROFILING_MUTEX};

                let should_profile = std::env::var("THAG_PROFILE").ok().is_some();
                eprintln!("should_profile={should_profile}");

            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use thag_profiler::{disable_profiling, finalize_profiling, init_profiling, ProfileType, PROFILING_MUTEX};
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
                use ::thag_profiler::{finalize_profiling, init_profiling, with_allocator, Allocator, PROFILING_MUTEX};

                let should_profile = with_allocator(Allocator::System, || {
                    std::env::var("THAG_PROFILE").ok().is_some()
                });

            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use ::thag_profiler::{disable_profiling, enable_profiling, finalize_profiling, init_profiling, profiled, with_allocator, Allocator, ProfileType, PROFILING_MUTEX};
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
                    finalize_profiling();  // Already uses with_allocator(Allocator::System... internally
                }
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Drop the profile explicitly at the end
                #profile_drop

                // Finalize profiling
                finalize_profiling();  // Already uses with_allocator(Allocator::System... internally
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

    // Verbosity is the price we pay for having to replicate the enum.
    let profile_type = match args.profile_type {
        Some(ProfileType::Both) => quote! {
            Some(ProfileType::Both)
        },
        Some(ProfileType::Memory) => quote! {
            Some(ProfileType::Memory)
        },
        Some(ProfileType::Time) => quote! {
            Some(ProfileType::Time)
        },
        None => quote! {
            None
        },
    };

    // let profile_type = args.profile_type;

    #[cfg(not(feature = "full_profiling"))]
    let wrapped_block = match args.mode {
        ProfilingMode::Runtime => quote! {
            let _guard = if should_profile {
                // Acquire the mutex to ensure only one instance can be profiling at a time
                Some(PROFILING_MUTEX.lock())
            } else {None};

            init_profiling(module_path!(), #profile_type);

            let maybe_profile = if should_profile {
                #profile_new
            } else {
                None
            };

            #wrapped_block
        },
        ProfilingMode::Enabled => quote! {
            // Acquire the mutex to ensure only one instance can be profiling at a time
            let _guard = PROFILING_MUTEX.lock();

            // Initialize profiling
            init_profiling(module_path!(), #profile_type);

            let profile = #profile_new;

            #wrapped_block
        },
        ProfilingMode::Disabled => quote! {
            #wrapped_block
        },
    };

    #[cfg(feature = "full_profiling")]
    let wrapped_block = match args.mode {
        ProfilingMode::Runtime => quote! {
            let _guard = with_allocator(Allocator::System, || {
                if should_profile {
                    // Acquire the mutex to ensure only one instance can be profiling at a time
                    Some(PROFILING_MUTEX.lock())
                } else {None}
            });

            init_profiling(module_path!(), #profile_type);  // Already uses with_allocator(Allocator::System... internally

            let maybe_profile = with_allocator(Allocator::System, || {
                if should_profile {
                    #profile_new
                } else {
                    None
                }
            });

            #wrapped_block
        },
        ProfilingMode::Enabled => quote! {
            // Acquire the mutex to ensure only one instance can be profiling at a time
            let _guard = with_allocator(Allocator::System, || {
                PROFILING_MUTEX.lock()
            });

            // Initialize profiling
            init_profiling(module_path!(), #profile_type);  // Already uses with_allocator(Allocator::System... internally

            let profile = with_allocator(Allocator::System, || {
                #profile_new
            });

            #wrapped_block
        },
        ProfilingMode::Disabled => quote! {
            #wrapped_block
        },
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
