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
    #[allow(dead_code)]
    None, // This variant is used in the codebase even though the diagnostic says otherwise
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
            #[allow(clippy::match_same_arms)]
            "both" => Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Both),
            },
            #[allow(clippy::match_same_arms)]
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

/// Detect if a function body appears to have been transformed by `tokio::main`
fn detect_tokio_main_expansion(body: &syn::Block) -> bool {
    // Look for patterns like: let body = async { ... }
    for stmt in &body.stmts {
        if let syn::Stmt::Local(local) = stmt {
            // Check if this is a "let body = ..." statement
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                if pat_ident.ident == "body" {
                    // Check if it's assigned an async expression
                    if let Some(init) = &local.init {
                        if let syn::Expr::Async(_) = &*init.expr {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Alternative approach: look for tokio runtime initialization
    let body_str = quote!(#body).to_string();
    body_str.contains("tokio::runtime")
        || body_str.contains("Runtime::new")
        || body_str.contains("let body = async") // Simpler string-based detection
}

#[allow(clippy::too_many_lines)]
pub fn enable_profiling_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(cfg!(feature = "time_profiling"));

    let args = parse_macro_input!(attr as ProfilingArgs);
    let input = parse_macro_input!(item as ItemFn);

    // Check if the function is explicitly async
    let is_explicitly_async = input.sig.asyncness.is_some();

    // Check if the function body appears to have been transformed by tokio::main
    let is_tokio_transformed = detect_tokio_main_expansion(&input.block);

    // Function is async if it's either explicitly marked async or shows signs of tokio transformation
    let is_async = is_explicitly_async || is_tokio_transformed;

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    // let body = quote!(#block);
    // eprintln!("body={body:#?}");
    // let is_async = if is_async { true } else {
    //     let
    // };

    for attr in attrs {
        assert_ne!(
            quote!(#attr).to_string().as_str(),
            "#[async_std :: main]",
            "#[async_std::main] if present must appear before #[enable_profiling] for correct expansion."
        );
        // eprintln!("attr={}", quote!(#attr));
        assert_ne!(
            quote!(#attr).to_string().as_str(),
            "#[tokio :: main]",
            "#[tokio::main] if present must appear before #[enable_profiling] for correct expansion."
        );
    }

    // let module_path = module_path!();
    let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    let profile_new = quote! {
        ::thag_profiler::Profile::new(None, Some(#fn_name_str), ::thag_profiler::get_global_profile_type(), #is_async, ::thag_profiler::is_detailed_memory(), file!(), None, None)
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
                use ::thag_profiler::{finalize_profiling, init_profiling, parse_env_profile_config, PROFILING_MUTEX};

                let should_profile = std::env::var("THAG_PROFILER").ok().is_some();
                eprintln!("should_profile={should_profile}");
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use thag_profiler::{disable_profiling, finalize_profiling, init_profiling, ProfileConfiguration, ProfileType, PROFILING_MUTEX};
            }
        }
        ProfilingMode::Disabled => {
            quote! {
                thag_profiler::enable_profiling(false, None).expect("Failed to disable profiling");
            }
        }
    };

    #[cfg(feature = "full_profiling")]
    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                use ::thag_profiler::{finalize_profiling, init_profiling, parse_env_profile_config, with_allocator, Allocator, PROFILING_MUTEX};

                let should_profile = with_allocator(Allocator::System, || {
                    std::env::var("THAG_PROFILER").ok().is_some()
                });

                with_allocator(Allocator::System, || {
                    eprintln!("should_profile={should_profile}");
                });
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use ::thag_profiler::{disable_profiling, enable_profiling, finalize_profiling, init_profiling, profiled, with_allocator, Allocator, ProfileConfiguration, ProfileType, PROFILING_MUTEX};
            }
        }
        ProfilingMode::Disabled => {
            quote! {
                thag_profiler::enable_profiling(false, None).expect("Failed to disable profiling");
            }
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

    let async_token = if is_async && !(fn_name == "main" && is_tokio_transformed) {
        quote!(async)
    } else {
        quote!()
    };

    let wrapped_block = if is_async && !(fn_name == "main" && is_tokio_transformed) {
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
        Some(ProfileType::None) | None => quote! {
            None
        },
    };

    #[cfg(not(feature = "full_profiling"))]
    let wrapped_block = match args.mode {
        ProfilingMode::Runtime => quote! {
            let _guard = if should_profile {
                // Acquire the mutex to ensure only one instance can be profiling at a time
                Some(PROFILING_MUTEX.lock())
            } else {None};

            if should_profile {
                // eprintln!("Calling init_profiling({}, Some({:?}))", module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));
                init_profiling(module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));
            }

            let maybe_profile = if should_profile {
                #profile_new
            } else {
                None
            };

            #wrapped_block
        },
        ProfilingMode::Enabled => {
            quote! {
                // Acquire the mutex to ensure only one instance can be profiling at a time
                let _guard = PROFILING_MUTEX.lock();

                // Initialize profiling
                let mut profile_config = ProfileConfiguration::default();
                profile_config.set_profile_type(#profile_type);
                init_profiling(module_path!(), profile_config);

                let profile = #profile_new;

                #wrapped_block
            }
        }
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

            if should_profile {
                // with_allocator(Allocator::System, || {
                //     eprintln!("Calling init_profiling({}, {:?})", module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));
                // });
                init_profiling(module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));   // Already uses with_allocator(Allocator::System... internally
            }

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
            let profile_config = with_allocator(Allocator::System, || {
                // ProfileConfiguration { profile_type: #profile_type, ..Default::default() };
                let mut profile_config = ProfileConfiguration::default();
                profile_config.set_profile_type(#profile_type);
                profile_config
            });
            init_profiling(module_path!(), profile_config);  // Already uses with_allocator(Allocator::System... internally

            let profile = with_allocator(Allocator::System, || {
                #profile_new
            });

            #wrapped_block
        },
        ProfilingMode::Disabled => quote! {
            enable_profiling(false, None).expect("Failed to disable profiling");

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
