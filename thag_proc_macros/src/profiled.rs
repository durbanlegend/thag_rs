#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;

use syn::{
    parse::{Parse, ParseStream},
    LitStr,
};

#[cfg(feature = "profiling")]
use quote::quote;

#[cfg(feature = "profiling")]
use syn::{
    parse_macro_input, Attribute, FnArg, Generics, ItemFn, ReturnType, Type, Visibility,
    WhereClause,
};

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
                // "trait_name" => {
                //     let lit: LitStr = input.parse()?;
                //     args.trait_name = Some(lit.value());
                // }
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
#[cfg(feature = "profiling")]
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
    // Generated profile name incorporating context (impl/trait/async/etc.)
    // profile_name: String,
    // /// Whether the function is asynchronous
    // is_async: bool,
    /// Whether the function is a method
    is_method: bool,
}

/// Determines if a function is a method by checking for:
/// 1. Explicit self parameter
/// 2. Return type of Self (including references to Self)
/// 3. Return type containing Self as a generic parameter (Result<Self>, Option<Self>, etc.)
/// 4. Location within an impl block (when available)
#[cfg(feature = "profiling")]
fn is_method(inputs: &[FnArg], output: &ReturnType) -> bool {
    // Check for self parameter (the most reliable indicator)
    let has_self_param = inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)));
    if has_self_param {
        return true;
    }

    // Check for Self return type (including references and wrapped types)
    let returns_self = match output {
        ReturnType::Type(_, ty) => {
            // Use our enhanced contains_self_type function
            contains_self_type(ty)
        }
        ReturnType::Default => false,
    };

    // Consider functions named "new" as methods even if they don't have self parameters
    // This helps with constructor methods like `fn new() -> Result<Self, Error>`
    if returns_self {
        return true;
    }

    false
}

/// Recursively checks if a type contains Self
#[cfg(feature = "profiling")]
fn contains_self_type(ty: &Type) -> bool {
    match ty {
        // Handle reference types (&Self, &'static Self, etc.)
        Type::Reference(type_reference) => contains_self_type(&type_reference.elem),

        // Handle plain Self or paths containing Self (like module::Self)
        Type::Path(type_path) => {
            // Check if any path segment is exactly "Self"
            let has_self_segment = type_path
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "Self");

            if has_self_segment {
                return true;
            }

            // Check for generic types that might contain Self (like Result<Self>)
            type_path.path.segments.iter().any(|segment| {
                // Check if this segment has generic parameters
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    // Examine each generic argument
                    args.args.iter().any(|arg| {
                        if let syn::GenericArgument::Type(inner_type) = arg {
                            // Recursively check if the generic type contains Self
                            contains_self_type(inner_type)
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            })
        }

        // Handle tuple types like (Self, T)
        Type::Tuple(tuple) => tuple.elems.iter().any(contains_self_type),

        // Handle array types like [Self; N]
        Type::Array(array) => contains_self_type(&array.elem),

        // Handle pointer types like *const Self
        Type::Ptr(ptr) => contains_self_type(&ptr.elem),

        // Handle slices like &[Self]
        Type::Slice(slice) => contains_self_type(&slice.elem),

        // Handle other type variants
        _ => false,
    }
}

#[cfg(not(feature = "profiling"))]
pub fn profiled_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Always check the feature flag at runtime to handle when the proc macro
    // is compiled with the feature but used without it
    item
}

#[cfg(feature = "profiling")]
pub fn profiled_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ProfileArgs);
    let item_clone = item.clone();
    let input = parse_macro_input!(item_clone as ItemFn);

    let fn_name = &input.sig.ident;

    if fn_name.to_string().as_str() == "main" {
        eprintln!("`main `function may only be profiled through #[enable_function] attribute - ignoring #[profiled] attribute");
        return item;
    }

    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    // let generics = &input.sig.generics;
    let is_async = input.sig.asyncness.is_some();

    // Convert Punctuated to slice for is_method
    let input_args: Vec<_> = inputs.iter().cloned().collect();
    // Determine if this is a method
    let is_method = is_method(&input_args, output);

    // Debugging aid - uncomment to see method detection information
    // This will show up in the compiler output and then stop compilation
    // if fn_name == "new" {
    //     let return_type = match output {
    //         ReturnType::Type(_, ty) => format!("{:?}", ty),
    //         ReturnType::Default => "()".to_string(),
    //     };
    //     return syn::Error::new(
    //         input.sig.span(),
    //         format!("DEBUG: fn_name={}, is_method={}, return_type={}", fn_name, is_method, return_type)
    //     ).to_compile_error().into();
    // }

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

    // Generate profile name
    // We no longer need to generate a profile name string to pass to Profile::new
    // Just to identify the function in the attribute macro for debugging
    // let profile_name =
    //     generate_profile_name(fn_name, is_method, &args /*, &type_params, is_async */);

    let ctx = FunctionContext {
        vis: &input.vis,
        fn_name,
        generics: &input.sig.generics,
        inputs: &input.sig.inputs,
        output: &input.sig.output,
        where_clause: input.sig.generics.where_clause.as_ref(),
        body: &input.block,
        attrs: &input.attrs,
        // profile_name,
        // is_async,
        is_method,
    };

    if is_async {
        generate_async_wrapper(&ctx, args.profile_type.as_ref())
    } else {
        generate_sync_wrapper(&ctx, args.profile_type.as_ref())
    }
    .into()
}

// #[allow(dead_code)]
// fn generate_profile_name(
//     fn_name: &syn::Ident,
//     is_method: bool,
//     args: &ProfileArgs,
//     // type_params: &[String],
//     // is_async: bool,
// ) -> String {
//     let mut parts = Vec::new();

//     if is_method {
//         if let Some(impl_type) = &args.imp {
//             parts.push(impl_type.to_string());
//         }
//     }

//     // Add function name
//     parts.push(fn_name.to_string());

//     // Use a compile error to display debug information
//     // This will show up in the compiler output and then stop compilation
//     // To enable, uncomment the following line:
//     // return syn::Error::new_spanned(
//     //    fn_name,
//     //    format!("DEBUG: Profile name: {}", parts.join("::"))
//     // ).to_compile_error().into();

//     parts.join("::")
// }

#[cfg(feature = "profiling")]
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
        attrs,
        // profile_name,
        // is_async,
        is_method,
    }: &FunctionContext<'_> = ctx;

    let profile_type = resolve_profile_type(profile_type);
    // let maybe_fn_name = format!(r#"Some("{fn_name}")"#);
    let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    quote! {

        #(#attrs)*
        #vis fn #fn_name #generics (#inputs) #output #where_clause {

            // We pass None for the name as we rely on the backtrace to identify the function
            let _profile = ::thag_profiler::Profile::new(None, Some(#fn_name_str), #profile_type, false, #is_method);
            #body
        }
    }
}

#[cfg(feature = "profiling")]
fn resolve_profile_type(profile_type: Option<&ProfileTypeOverride>) -> proc_macro2::TokenStream {
    match profile_type {
        Some(ProfileTypeOverride::Global) | None => {
            quote!(::thag_profiler::profiling::get_global_profile_type())
        }
        Some(ProfileTypeOverride::Time) => quote!(::thag_profiler::ProfileType::Time),
        Some(ProfileTypeOverride::Memory) => quote!(::thag_profiler::ProfileType::Memory),
        Some(ProfileTypeOverride::Both) => quote!(::thag_profiler::ProfileType::Both),
    }
}

#[cfg(feature = "profiling")]
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
        attrs,
        // profile_name,
        // is_async,
        is_method,
    } = ctx;

    let profile_type = resolve_profile_type(profile_type);
    // let maybe_fn_name = format!(r#"Some("{fn_name}")"#);
    let fn_name_str = fn_name.to_string(); // format!("{fn_name}");

    // let is_method = ctx.is_method;

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
                        this._profile.take();
                    }
                    result
                }
            }

            // eprintln!("From generate_async_wrapper: profile_name={}", #profile_name);

            let future = async #body;
            ProfiledFuture {
                inner: future,
                _profile: ::thag_profiler::Profile::new(None, Some(#fn_name_str), #profile_type, true, #is_method),
            }.await
        }
    }
}
