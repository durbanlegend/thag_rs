#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr,
};

use syn::{parse_macro_input, ItemFn};

use syn::{Attribute, FnArg, Generics, ReturnType, Visibility, WhereClause};

/// Configuration for `profiled` attribute macro
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
struct ProfileArgs {
    /// The implementing type (e.g., "`MyStruct`") - kept for backwards compatibility
    imp: Option<String>,
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

impl Parse for ProfileArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        // Handle empty case
        if input.is_empty() {
            args.global = true; // Default to global if no args specified
            return Ok(args);
        }

        // First attempt to parse as named parameters
        if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;
                let _: syn::Token![=] = input.parse()?;

                let ident_str = ident.to_string();
                match ident_str.as_str() {
                    "imp" => {
                        let lit: LitStr = input.parse()?;
                        args.imp = Some(lit.value());
                    }
                    // For backward compatibility
                    "detailed_memory" => {
                        let lit: syn::LitBool = input.parse()?;
                        args.mem_detail = lit.value;
                    }
                    // For backward compatibility
                    "profile_type" => {
                        let lit: LitStr = input.parse()?;
                        match lit.value().as_str() {
                            "time" => args.time = true,
                            "memory" => args.mem_summary = true,
                            "both" => args.both = true,
                            "global" => args.global = true,
                            _ => return Err(syn::Error::new(lit.span(), "invalid profile type")),
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("unknown parameter: {ident_str}"),
                        ));
                    }
                }

                if !input.is_empty() {
                    let _: syn::Token![,] = input.parse()?;
                }
            }
        } else {
            // Parse as a list of flags
            let mut first = true;

            while !input.is_empty() {
                if !first {
                    let _: syn::Token![,] = input.parse()?;
                }
                first = false;

                // Check for imp parameter with a different syntax
                if input.peek(syn::Ident) && input.peek2(syn::Token![=]) {
                    let ident: syn::Ident = input.parse()?;
                    if ident == "imp" {
                        let _: syn::Token![=] = input.parse()?;
                        let lit: LitStr = input.parse()?;
                        args.imp = Some(lit.value());
                        continue;
                    }
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unexpected parameter: {ident}"),
                    ));
                }

                // Parse as flag
                let flag: syn::Ident = input.parse()?;
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
                            format!("unknown flag: {flag}"),
                        ));
                    }
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

pub fn profiled_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ProfileArgs);
    let item_clone = item.clone();
    let input = parse_macro_input!(item_clone as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();

    if fn_name_str == "main" {
        eprintln!("`main` function may only be profiled through #[enable_function] attribute - ignoring #[profiled] attribute");
        return item;
    }

    let is_async = input.sig.asyncness.is_some();

    // Determine profile type based on flags
    let profile_type = if args.both || (args.time && (args.mem_summary || args.mem_detail)) {
        quote! { ::thag_profiler::ProfileType::Both }
    } else if args.time {
        quote! { ::thag_profiler::ProfileType::Time }
    } else if args.mem_summary || args.mem_detail {
        quote! { ::thag_profiler::ProfileType::Memory }
    } else {
        // Default to global
        quote! { ::thag_profiler::get_global_profile_type() }
    };

    // Determine if detailed memory profiling is enabled
    let detailed_memory = args.mem_detail;

    // Check if this is a test function by name or by explicit flag
    let is_test_fn = args.test || fn_name_str.ends_with("_test");

    #[cfg(not(feature = "full_profiling"))]
    let profile_new = quote! {
        ::thag_profiler::Profile::new(None, Some(#fn_name_str), #profile_type, #is_async, #detailed_memory, file!(), None, None)
    };

    #[cfg(feature = "full_profiling")]
    let profile_new = quote! {
        ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
            ::thag_profiler::Profile::new(None, Some(#fn_name_str), #profile_type, #is_async, #detailed_memory, file!(), None, None)
        })
    };

    #[cfg(not(feature = "full_profiling"))]
    let profile_drop = quote! {
        drop(profile);
    };

    #[cfg(feature = "full_profiling")]
    let profile_drop = quote! {
        ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
            drop(profile);
        });
    };

    let ctx = FunctionContext {
        vis: &input.vis,
        fn_name,
        generics: &input.sig.generics,
        inputs: &input.sig.inputs,
        output: &input.sig.output,
        where_clause: input.sig.generics.where_clause.as_ref(),
        body: &input.block,
        attrs: &input.attrs,
        profile_new,
        profile_drop,
        is_test_fn,
    };

    if is_async {
        generate_async_wrapper(&ctx)
    } else {
        generate_sync_wrapper(&ctx)
    }
    .into()
}

fn generate_sync_wrapper(ctx: &FunctionContext) -> proc_macro2::TokenStream {
    let FunctionContext {
        vis,
        fn_name,
        generics,
        inputs,
        output,
        where_clause,
        body,
        attrs,
        profile_new,
        profile_drop,
        is_test_fn: _,
    }: &FunctionContext<'_> = ctx;

    quote! {
        #(#attrs)*
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            // We pass None for the name as we rely on the backtrace to identify the function
            let profile = #profile_new;
            let result = { #body };

            #profile_drop

            result
        }
    }
}

fn generate_async_wrapper(ctx: &FunctionContext) -> proc_macro2::TokenStream {
    let FunctionContext {
        vis,
        fn_name,
        generics,
        inputs,
        output,
        where_clause,
        body,
        attrs,
        profile_new,
        profile_drop,
        is_test_fn,
    } = ctx;

    // For test functions or functions with _test suffix, create a clone
    // to make profile available inside the function body
    let profile_setup = if *is_test_fn {
        quote! {
            let profile = #profile_new;
            let profile_for_future = profile.clone();
        }
    } else {
        quote! {
            let profile = #profile_new;
        }
    };

    // Choose the right profile for the future
    let future_profile = if *is_test_fn {
        quote! { profile_for_future }
    } else {
        quote! { profile }
    };

    // If this is a test function, add a debug message
    let fn_name_str = fn_name.to_string();
    let debug_msg = if *is_test_fn {
        quote! {
            ::thag_profiler::debug_log!("Using cloned profile for test function: {}", #fn_name_str);
        }
    } else {
        quote! {}
    };

    quote! {
        #(#attrs)*
        #vis async fn #fn_name #generics (#inputs) #output #where_clause {
            use std::future::Future;
            use std::pin::Pin;
            use std::task::{Context, Poll};

            struct ProfiledFuture<F> {
                inner: F,
                _profile: Option<::thag_profiler::Profile>,
            }

            impl<F: Future> Future for ProfiledFuture<F> {
                type Output = F::Output;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let this = unsafe { self.as_mut().get_unchecked_mut() };
                    let result = unsafe { Pin::new_unchecked(&mut this.inner) }.poll(cx);
                    if result.is_ready() {
                        // Take the profile out so we can explicitly drop it with the System allocator
                        if let Some(profile) = this._profile.take() {
                            #profile_drop
                        }
                    }
                    result
                }
            }

            #debug_msg
            #profile_setup

            let future = async #body;
            ProfiledFuture {
                inner: future,
                _profile: #future_profile,
            }.await
        }
    }
}

/// Context for generating profiled function wrappers
///
/// This struct contains all the necessary components to generate either a synchronous
/// or asynchronous function wrapper with profiling capabilities.
#[derive(Debug)]
struct FunctionContext<'a> {
    /// Function visibility (pub, pub(crate), etc.)
    vis: &'a Visibility,
    /// Function name identifier
    fn_name: &'a syn::Ident,
    /// Generic parameters including lifetimes and type parameters
    generics: &'a Generics,
    /// Function parameters
    inputs: &'a syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
    /// Function return type
    output: &'a ReturnType,
    /// Optional where clause for generic constraints
    where_clause: Option<&'a WhereClause>,
    /// Function body
    body: &'a syn::Block,
    attrs: &'a Vec<Attribute>,
    /// Profile instantiation, avoiding allocation tracking if memory profiling
    profile_new: proc_macro2::TokenStream,
    /// Profile drop, avoiding allocation tracking if memory profiling
    profile_drop: proc_macro2::TokenStream,
    /// Is this a test function (either by name convention or explicit flag)
    is_test_fn: bool,
}
