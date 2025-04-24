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
#[derive(Default)]
struct ProfileArgs {
    /// The implementing type (e.g., "`MyStruct`") - kept for backwards compatibility
    /// but not used in `Profile::new` any more (backtrace provides this information)
    imp: Option<String>,
    // The trait being implemented (e.g., "`Display`") - removed as unused
    // trait_name: Option<String>,
    /// Explicit profile type override
    profile_type: Option<ProfileTypeOverride>,
    /// Whether to enable detailed memory profiling for this function
    detailed_memory: bool,
}

impl Parse for ProfileArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: syn::Token![=] = input.parse()?;

            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "imp" => {
                    let lit: LitStr = input.parse()?;
                    args.imp = Some(lit.value());
                }
                "detailed_memory" => {
                    let lit: syn::LitBool = input.parse()?;
                    args.detailed_memory = lit.value;
                }
                _ => {
                    args.profile_type = Some(match ident_str.as_str() {
                        "global" => ProfileTypeOverride::Global,
                        "time" => ProfileTypeOverride::Time,
                        "memory" => ProfileTypeOverride::Memory,
                        "both" => ProfileTypeOverride::Both,
                        _ => return Err(syn::Error::new(ident.span(), "invalid profile type")),
                    });
                }
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(args)
    }
}

/// Explicit profile type configuration
#[derive(Debug, Clone)]
enum ProfileTypeOverride {
    /// Use the global profile type set in `enable_profiling`
    Global,
    /// Override with specific type
    Time,
    Memory,
    Both,
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
    // Profile instantiation, avoiding allocation tracking if memory profiling
    profile_new: proc_macro2::TokenStream,
    // /// Whether the function is asynchronous
    // is_async: bool,
}

pub fn profiled_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ProfileArgs);
    let item_clone = item.clone();
    let input = parse_macro_input!(item_clone as ItemFn);

    let fn_name = &input.sig.ident;

    if fn_name.to_string().as_str() == "main" {
        eprintln!("`main `function may only be profiled through #[enable_function] attribute - ignoring #[profiled] attribute");
        return item;
    }

    // let inputs = &input.sig.inputs;
    // let output = &input.sig.output;
    // let generics = &input.sig.generics;
    let is_async = input.sig.asyncness.is_some();

    // let input_args: Vec<_> = inputs.iter().cloned().collect();

    // Get generic parameters
    // let type_params: Vec<_> = generics
    //     .params
    //     .iter()
    //     .map(|param| match param {
    //         syn::GenericParam::Type(t) => t.ident.to_string(),
    //         syn::GenericParam::Lifetime(l) => l.lifetime.to_string(),
    //         syn::GenericParam::Const(c) => c.ident.to_string(),
    //     })
    //     .collect();

    // let module_path = module_path!();
    let fn_name_str = fn_name.to_string();
    let detailed_memory = args.detailed_memory;

    #[cfg(not(feature = "full_profiling"))]
    let profile_new = quote! {
        ::thag_profiler::Profile::new(None, Some(#fn_name_str), ::thag_profiler::get_global_profile_type(), #is_async, #detailed_memory, file!(), None, None)
    };

    #[cfg(feature = "full_profiling")]
    let profile_new = quote! {
        ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
            ::thag_profiler::Profile::new(None, Some(#fn_name_str), ::thag_profiler::get_global_profile_type(), #is_async, #detailed_memory, file!(), None, None)
        })
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
        // is_async,
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
        // is_async,
    }: &FunctionContext<'_> = ctx;

    // let profile_type = resolve_profile_type(profile_type);
    // let maybe_fn_name = format!(r#"Some("{fn_name}")"#);
    // let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    quote! {

        #(#attrs)*
        #vis fn #fn_name #generics (#inputs) #output #where_clause {

            // We pass None for the name as we rely on the backtrace to identify the function
            let profile = #profile_new;
            let result = { #body };

            ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
                drop(profile);
            });

            result
        }
    }
}

// fn resolve_profile_type(profile_type: Option<&ProfileTypeOverride>) -> proc_macro2::TokenStream {
//     match profile_type {
//         Some(ProfileTypeOverride::Global) | None => {
//             quote!(::thag_profiler::profiling::get_global_profile_type())
//         }
//         Some(ProfileTypeOverride::Time) => quote!(::thag_profiler::ProfileType::Time),
//         Some(ProfileTypeOverride::Memory) => quote!(::thag_profiler::ProfileType::Memory),
//         Some(ProfileTypeOverride::Both) => quote!(::thag_profiler::ProfileType::Both),
//     }
// }

fn generate_async_wrapper(
    ctx: &FunctionContext,
    // profile_type: Option<&ProfileTypeOverride>,
) -> proc_macro2::TokenStream {
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
        // is_async,
    } = ctx;

    // let profile_type = resolve_profile_type(profile_type);
    // let maybe_fn_name = format!(r#"Some("{fn_name}")"#);
    // let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

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
                            ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
                                drop(profile);
                            });
                        }
                    }
                    result
                }
            }

            // eprintln!("From generate_async_wrapper: profile_name={}", #profile_name);

            let future = async #body;
            ProfiledFuture {
                inner: future,
                _profile: #profile_new,
            }.await
        }
    }
}
