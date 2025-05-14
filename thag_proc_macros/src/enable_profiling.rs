#![allow(clippy::module_name_repetitions)]
use std::result::Result;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ItemFn, Token,
};

#[derive(Default, PartialEq, Eq)]
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

// Function-level profiling arguments, similar to #[profiled] macro
#[derive(Default)]
struct FunctionProfileArgs {
    /// Flag for time profiling
    time: bool,
    /// Flag for memory summary profiling
    mem_summary: bool,
    /// Flag for detailed memory profiling
    mem_detail: bool,
    /// Flag for both time and memory profiling
    both: bool,
    /// Flag for using global profiling settings
    global: bool,
    /// Flag for creating profile clone for testing
    test: bool,
}

impl Parse for FunctionProfileArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut args = Self::default();

        // Handle empty case
        if input.is_empty() {
            args.global = true; // Default to global if no args specified
            return Ok(args);
        }

        // Parse as a list of flags
        let flags = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

        for flag in flags {
            match flag.to_string().as_str() {
                "time" => args.time = true,
                "mem_summary" => args.mem_summary = true,
                "mem_detail" => args.mem_detail = true,
                "both" => args.both = true,
                "global" => args.global = true,
                "test" => args.test = true,
                _ => {
                    return Err(syn::Error::new(
                        flag.span(),
                        format!("unknown function profiling flag: {flag}"),
                    ));
                }
            }
        }

        // If no profiling type was specified, default to global
        if !args.time && !args.mem_summary && !args.mem_detail && !args.both && !args.global {
            args.global = true;
        }

        Ok(args)
    }
}

/// Configuration for `enable_profiling` attribute macro
#[derive(Default)]
struct ProfilingArgs {
    mode: ProfilingMode,
    profile_type: Option<ProfileType>,
    function_args: Option<FunctionProfileArgs>,
}

impl Parse for ProfilingArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // Empty input means use default
        #[cfg(not(feature = "full_profiling"))]
        if input.is_empty() {
            return Ok(Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Time),
                function_args: None,
            });
        }

        #[cfg(feature = "full_profiling")]
        if input.is_empty() {
            return Ok(Self {
                mode: ProfilingMode::Enabled,
                profile_type: Some(ProfileType::Both),
                function_args: None,
            });
        }

        let mut result = Self::default();
        let mut mode_set = false;

        // Parse as a comma-separated list of parameters
        while !input.is_empty() {
            if !input.peek(Ident) {
                return Err(syn::Error::new(input.span(), "Expected identifier"));
            }

            let ident: Ident = input.parse()?;
            let param_name = ident.to_string();

            if param_name == "function" {
                // Parse function-level parameters in parentheses
                let content;
                syn::parenthesized!(content in input);
                result.function_args = Some(content.parse()?);
            } else {
                // Handle global parameters
                match param_name.as_str() {
                    "no" => {
                        result.mode = ProfilingMode::Disabled;
                        mode_set = true;
                    }
                    "runtime" => {
                        result.mode = ProfilingMode::Runtime;
                        mode_set = true;
                    }
                    "both" => {
                        result.profile_type = Some(ProfileType::Both);
                        if !mode_set {
                            result.mode = ProfilingMode::Enabled;
                            mode_set = true;
                        }
                    }
                    "yes" => {
                        result.mode = ProfilingMode::Enabled;
                        #[cfg(feature = "full_profiling")]
                        {
                            result.profile_type = Some(ProfileType::Both);
                        }
                        #[cfg(not(feature = "full_profiling"))]
                        {
                            result.profile_type = Some(ProfileType::Time);
                        }
                        mode_set = true;
                    }
                    "memory" => {
                        result.profile_type = Some(ProfileType::Memory);
                        if !mode_set {
                            result.mode = ProfilingMode::Enabled;
                            mode_set = true;
                        }
                    }
                    "time" => {
                        result.profile_type = Some(ProfileType::Time);
                        if !mode_set {
                            result.mode = ProfilingMode::Enabled;
                            mode_set = true;
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("Unknown parameter: {param_name}. Expected 'memory', 'time', 'both', 'runtime', 'yes', 'no' or 'function(...)'")
                        ));
                    }
                }
            }

            // Check for comma separator unless we're at the end
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(result)
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

    // #[enabled(no)] specified
    if args.mode == ProfilingMode::Disabled {
        return item.into();
    }

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

    for attr in attrs {
        assert_ne!(
            quote!(#attr).to_string().as_str(),
            "#[async_std :: main]",
            "#[async_std::main] if present must appear before #[enable_profiling] for correct expansion."
        );
        assert_ne!(
            quote!(#attr).to_string().as_str(),
            "#[tokio :: main]",
            "#[tokio::main] if present must appear before #[enable_profiling] for correct expansion."
        );
    }

    let fn_name_str = fn_name.to_string();

    // Determine if detailed memory profiling is enabled from function args
    let is_detailed_memory = if let Some(fn_args) = &args.function_args {
        fn_args.mem_detail
    } else {
        false
    };

    // Function profiling type
    #[allow(unused_variables)]
    let function_profile_type = if let Some(fn_args) = &args.function_args {
        #[cfg(feature = "full_profiling")]
        let profile_type =
            if fn_args.both || (fn_args.time && (fn_args.mem_summary || fn_args.mem_detail)) {
                quote! { ::thag_profiler::ProfileType::Both }
            } else if fn_args.time {
                quote! { ::thag_profiler::ProfileType::Time }
            } else if fn_args.mem_summary || fn_args.mem_detail {
                quote! { ::thag_profiler::ProfileType::Memory }
            } else {
                // Default to global
                quote! { ::thag_profiler::get_global_profile_type() }
            };

        // When not using full_profiling, always use Time regardless of memory settings
        #[cfg(not(feature = "full_profiling"))]
        let profile_type = quote! { ::thag_profiler::ProfileType::Time };

        profile_type
    } else {
        // Default to global profile type
        quote! { ::thag_profiler::get_global_profile_type() }
    };

    let profile_new = quote! {
        ::thag_profiler::Profile::new(None, Some(#fn_name_str), #function_profile_type, #is_async, #is_detailed_memory, file!(), None, None)
    };

    #[cfg(not(feature = "full_profiling"))]
    let profile_drop = quote! {
        drop(profile);
    };

    #[cfg(feature = "full_profiling")]
    let profile_drop = quote! {
        with_sys_alloc(|| {
            drop(profile);
        });
    };

    #[cfg(not(feature = "full_profiling"))]
    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                use ::thag_profiler::{finalize_profiling, init_profiling, parse_env_profile_config, PROFILING_MUTEX};

                let should_profile = std::env::var("THAG_PROFILER").ok().is_some();
                // eprintln!("should_profile={should_profile}");
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use thag_profiler::{disable_profiling, finalize_profiling, init_profiling, ProfileConfiguration, ProfileType, PROFILING_MUTEX};
            }
        }
        _ => {
            quote! {}
        }
    };

    #[cfg(feature = "full_profiling")]
    let profile_init = match args.mode {
        ProfilingMode::Runtime => {
            quote! {
                use ::thag_profiler::{finalize_profiling, init_profiling, parse_env_profile_config, with_sys_alloc, Allocator, PROFILING_MUTEX};

                let should_profile = with_sys_alloc(|| {
                    std::env::var("THAG_PROFILER").ok().is_some()
                });

                // with_sys_alloc(|| {
                //     eprintln!("should_profile={should_profile}");
                // });
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                use ::thag_profiler::{disable_profiling, finalize_profiling, init_profiling, profiled, with_sys_alloc, Allocator, ProfileConfiguration, ProfileType, PROFILING_MUTEX};
            }
        }
        _ => {
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
                    finalize_profiling();  // Already uses with_sys_alloc(... internally
                }
            }
        }
        ProfilingMode::Enabled => {
            quote! {
                // Drop the profile explicitly at the end
                #profile_drop

                // Finalize profiling
                finalize_profiling();  // Already uses with_sys_alloc(... internally
            }
        }
        _ => {
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
        _ => unreachable!(),
    };

    #[cfg(feature = "full_profiling")]
    let wrapped_block = match args.mode {
        ProfilingMode::Runtime => quote! {
            let _guard = with_sys_alloc(|| {
                if should_profile {
                    // Acquire the mutex to ensure only one instance can be profiling at a time
                    Some(PROFILING_MUTEX.lock())
                } else {None}
            });

            if should_profile {
                // with_sys_alloc(|| {
                //     eprintln!("Calling init_profiling({}, {:?})", module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));
                // });
                init_profiling(module_path!(), parse_env_profile_config().expect("Error parsing environment variable THAG_PROFILER"));   // Already uses with_sys_alloc(... internally
            }

            let maybe_profile = with_sys_alloc(|| {
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
            let _guard = with_sys_alloc(|| {
                PROFILING_MUTEX.lock()
            });

            // Initialize profiling
            let profile_config = with_sys_alloc(|| {
                // ProfileConfiguration { profile_type: #profile_type, ..Default::default() };
                let mut profile_config = ProfileConfiguration::default();
                profile_config.set_profile_type(#profile_type);
                profile_config
            });
            init_profiling(module_path!(), profile_config);  // Already uses with_sys_alloc(... internally

            let profile = with_sys_alloc(|| {
                #profile_new
            });

            #wrapped_block
        },
        _ => unreachable!(),
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
