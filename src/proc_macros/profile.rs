#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, Generics, ItemFn, LitStr, ReturnType, Type, Visibility, WhereClause,
};

/// Configuration for profile attribute macro
#[derive(Default)]
struct ProfileArgs {
    /// The implementing type (e.g., "`MyStruct`")
    imp: Option<String>,
    /// The trait being implemented (e.g., "`Display`")
    trait_: Option<String>,
    /// Explicit profile type override
    profile_type: Option<ProfileTypeOverride>,
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
    /// Generated profile name incorporating context (impl/trait/async/etc.)
    profile_name: String,
}

impl Parse for ProfileArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: syn::Token![=] = input.parse()?;

            match ident.to_string().as_str() {
                "imp" => {
                    let lit: LitStr = input.parse()?;
                    args.imp = Some(lit.value());
                }
                "trait_" => {
                    let lit: LitStr = input.parse()?;
                    args.trait_ = Some(lit.value());
                }
                "profile_type" => {
                    let lit: LitStr = input.parse()?;
                    args.profile_type = Some(match lit.value().as_str() {
                        "global" => ProfileTypeOverride::Global,
                        "time" => ProfileTypeOverride::Time,
                        "memory" => ProfileTypeOverride::Memory,
                        "both" => ProfileTypeOverride::Both,
                        _ => return Err(syn::Error::new(lit.span(), "invalid profile type")),
                    });
                }
                _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(args)
    }
}

/// Determines if a function is a method by checking for:
/// 1. Explicit self parameter
/// 2. Return type of Self (including references to Self)
/// 3. Location within an impl block (when available)
fn is_method(inputs: &[FnArg], output: &ReturnType) -> bool {
    // Check for self parameter
    let has_self_param = inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)));
    if has_self_param {
        return true;
    }

    // Check for Self return type (including references to Self)
    match output {
        ReturnType::Type(_, ty) => contains_self_type(ty),
        ReturnType::Default => false,
    }
}

/// Recursively checks if a type contains Self
fn contains_self_type(ty: &Type) -> bool {
    match ty {
        // Handle reference types (&Self, &'static Self, etc.)
        Type::Reference(type_reference) => contains_self_type(&type_reference.elem),

        // Handle plain Self
        Type::Path(type_path) => type_path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Self"),

        // Handle other type variants if needed
        _ => false,
    }
}

pub fn profile_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ProfileArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let is_async = input.sig.asyncness.is_some();

    // Convert Punctuated to slice for is_method
    let input_args: Vec<_> = inputs.iter().cloned().collect();
    // Determine if this is a method
    // eprintln!("fn_name={fn_name}");
    let is_method = is_method(&input_args, output);

    // Get generic parameters
    let type_params: Vec<_> = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(t) => t.ident.to_string(),
            syn::GenericParam::Lifetime(l) => l.lifetime.to_string(),
            syn::GenericParam::Const(c) => c.ident.to_string(),
        })
        .collect();

    // Generate profile name
    let profile_name = generate_profile_name(fn_name, is_method, &args, &type_params, is_async);

    let ctx = FunctionContext {
        vis: &input.vis,
        fn_name,
        generics: &input.sig.generics,
        inputs: &input.sig.inputs,
        output: &input.sig.output,
        where_clause: input.sig.generics.where_clause.as_ref(),
        body: &input.block,
        profile_name,
    };

    if is_async {
        generate_async_wrapper(&ctx, args.profile_type.as_ref())
    } else {
        generate_sync_wrapper(&ctx, args.profile_type.as_ref())
    }
    .into()
}

fn generate_profile_name(
    fn_name: &syn::Ident,
    is_method: bool,
    args: &ProfileArgs,
    type_params: &[String],
    is_async: bool,
) -> String {
    let mut parts = Vec::new();

    // Add async prefix if applicable
    if is_async {
        parts.push("async".to_string());
    }

    // Add context (impl/trait/fn)
    if is_method {
        if let Some(trait_name) = &args.trait_ {
            parts.push(format!("trait::{trait_name}"));
            if let Some(impl_type) = &args.imp {
                parts.push(format!("impl::{impl_type}"));
            }
        } else if let Some(impl_type) = &args.imp {
            parts.push(format!("impl::{impl_type}"));
        } else {
            parts.push("method".to_string());
        }
    } else {
        parts.push("fn".to_string());
    }

    // Add function name
    parts.push(fn_name.to_string());

    // Add generic parameters if any
    if !type_params.is_empty() {
        parts.push(format!("<{}>", type_params.join(",")));
    }

    parts.join("::")
}

fn generate_sync_wrapper(
    ctx: &FunctionContext,
    profile_type: Option<&ProfileTypeOverride>,
) -> proc_macro2::TokenStream {
    let FunctionContext {
        vis,
        fn_name,
        generics,
        inputs,
        output,
        where_clause,
        body,
        profile_name,
    } = ctx;

    let profile_type = resolve_profile_type(profile_type);

    quote! {
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            let _profile = crate::Profile::new(#profile_name, #profile_type);
            #body
        }
    }
}

fn resolve_profile_type(profile_type: Option<&ProfileTypeOverride>) -> proc_macro2::TokenStream {
    match profile_type {
        Some(ProfileTypeOverride::Global) | None => {
            quote!(crate::profiling::get_global_profile_type())
        }
        Some(ProfileTypeOverride::Time) => quote!(crate::ProfileType::Time),
        Some(ProfileTypeOverride::Memory) => quote!(crate::ProfileType::Memory),
        Some(ProfileTypeOverride::Both) => quote!(crate::ProfileType::Both),
    }
}

fn generate_async_wrapper(
    ctx: &FunctionContext,
    profile_type: Option<&ProfileTypeOverride>,
) -> proc_macro2::TokenStream {
    let FunctionContext {
        vis,
        fn_name,
        generics,
        inputs,
        output,
        where_clause,
        body,
        profile_name,
    } = ctx;

    let profile_type = resolve_profile_type(profile_type);

    quote! {
        #vis async fn #fn_name #generics (#inputs) #output #where_clause {
            use std::future::Future;
            use std::pin::Pin;
            use std::task::{Context, Poll};

            struct ProfiledFuture<F> {
                inner: F,
                _profile: Option<crate::Profile>,
            }

            impl<F: Future> Future for ProfiledFuture<F> {
                type Output = F::Output;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let this = unsafe { self.as_mut().get_unchecked_mut() };
                    let result = unsafe { Pin::new_unchecked(&mut this.inner) }.poll(cx);
                    if result.is_ready() {
                        this._profile.take();
                    }
                    result
                }
            }

            let future = async #body;
            ProfiledFuture {
                inner: future,
                _profile: crate::Profile::new(#profile_name, #profile_type),
            }.await
        }
    }
}
