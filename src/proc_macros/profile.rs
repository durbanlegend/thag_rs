#![allow(clippy::module_name_repetitions)]
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, FnArg, Generics, Ident, ItemFn, LitStr, ReturnType, Type,
    Visibility, WhereClause,
};

/// Configuration for profile attribute macro
#[derive(Default)]
struct ProfileArgs {
    /// The implementing type (e.g., "`MyStruct`")
    imp: Option<String>,
    /// The trait being implemented (e.g., "`Display`")
    trait_name: Option<String>,
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
                "trait_name" => {
                    let lit: LitStr = input.parse()?;
                    args.trait_name = Some(lit.value());
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

// Track which functions have profiled versions
static PROFILED_FUNCTIONS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Register a function as having a profiled version
fn register_profiled_function(name: &str) {
    if let Ok(mut funcs) = PROFILED_FUNCTIONS.lock() {
        funcs.insert(name.to_string());
    }
}

/// Check if a function has a profiled version
pub fn is_profiled_function(name: &str) -> bool {
    PROFILED_FUNCTIONS
        .lock()
        .map(|funcs| funcs.contains(name))
        .unwrap_or(false)
}

pub fn profile_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Extract function details
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let asyncness = &sig.asyncness;
    let generics = &sig.generics;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let where_clause = &sig.generics.where_clause;
    let body = &input.block;

    // Check if this is a method
    let is_method = inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)));

    // Create a wrapper that simply adds profiling
    let profile_name = if is_method {
        format!("method::{}", fn_name)
    } else {
        format!("fn::{}", fn_name)
    };

    // Generate appropriate code based on whether this is async
    let expanded = if asyncness.is_some() {
        quote! {
            #vis #asyncness fn #fn_name #generics (#inputs) #output #where_clause {
                let _profile = crate::Profile::new(
                    #profile_name,
                    crate::profiling::get_global_profile_type(),
                    &[] // No parent stack - simpler approach
                );

                // Execute original function body directly
                async {
                    #body
                }.await
            }
        }
    } else {
        quote! {
            #vis fn #fn_name #generics (#inputs) #output #where_clause {
                let _profile = crate::Profile::new(
                    #profile_name,
                    crate::profiling::get_global_profile_type(),
                    &[] // No parent stack - simpler approach
                );

                // Execute original function body directly
                #body
            }
        }
    };

    expanded.into()
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
        if let Some(trait_name) = &args.trait_name {
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

    // Create profiled impl name
    let profiled_name = format_ident!("{}_profiled", fn_name);

    // Extract parameter names for forwarding to the profiled implementation
    let param_names = extract_param_names(inputs);

    // Transform the function body to call profiled versions
    let mut transformed_body = body.clone().clone();
    transform_function_calls(&mut transformed_body);

    let profile_type_expr = resolve_profile_type(profile_type);

    quote! {
        // Public function with original signature
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            // Call profiled implementation with empty stack
            #profiled_name(#(#param_names,)* &[])
        }

        // Hidden implementation with profile stack parameter
        fn #profiled_name #generics (#inputs, __profile_stack: &[u64]) #output #where_clause {
            // Create profile with parent stack
            let _profile = crate::Profile::new(#profile_name, #profile_type_expr, __profile_stack);

            // Create child stack for nested calls
            let mut child_stack = Vec::with_capacity(__profile_stack.len() + 1);
            child_stack.extend_from_slice(__profile_stack);
            if let Some(ref profile) = _profile {
                child_stack.push(profile.id());
            }

            // Execute transformed body
            #transformed_body
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

    // Create profiled impl name
    let profiled_name = format_ident!("{}_profiled", fn_name);

    // Extract parameter names
    let param_names = extract_param_names(inputs);

    // Transform function body to call profiled versions
    let mut transformed_body = body.clone().clone();
    transform_function_calls(&mut transformed_body);

    let profile_type_expr = resolve_profile_type(profile_type);

    quote! {
        // Public async function with original signature
        #vis async fn #fn_name #generics (#inputs) #output #where_clause {
            // Call profiled implementation with empty stack
            #profiled_name(#(#param_names,)* &[]).await
        }

        // Hidden async implementation with profile stack parameter
        async fn #profiled_name #generics (#inputs, __profile_stack: &[u64]) #output #where_clause {
            // Create profile with parent stack
            let _profile = crate::Profile::new(#profile_name, #profile_type_expr, __profile_stack);

            // Create child stack for nested calls
            let mut child_stack = Vec::with_capacity(__profile_stack.len() + 1);
            child_stack.extend_from_slice(__profile_stack);
            if let Some(ref profile) = _profile {
                child_stack.push(profile.id());
            }

            // Execute transformed body
            #transformed_body
        }
    }
}

use std::{collections::HashSet, sync::Mutex};
use syn::{visit_mut::VisitMut, Expr, ExprCall, Path};

struct ProfileCallTransformer;

impl VisitMut for ProfileCallTransformer {
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        // First visit nested expressions
        syn::visit_mut::visit_expr_call_mut(self, call);

        // Check if this is a call to a profiled function
        if let Expr::Path(expr_path) = &*call.func {
            if let Some(ident) = path_to_ident(&expr_path.path) {
                let func_name = ident.to_string();

                if is_profiled_function(&func_name) {
                    // Replace with profiled version
                    let profiled_name = format!("{}_profiled", func_name);
                    let profiled_ident = Ident::new(&profiled_name, ident.span());

                    // Create new path with profiled name
                    let mut new_path = expr_path.clone();
                    if let Some(last) = new_path.path.segments.last_mut() {
                        last.ident = profiled_ident;
                    }

                    // Replace function
                    call.func = Box::new(Expr::Path(new_path));

                    // Add child_stack argument
                    let child_stack_arg = parse_quote!(&child_stack);
                    call.args.push(child_stack_arg);
                }
            }
        }
    }
}

// Helper to get last ident from a path
fn path_to_ident(path: &Path) -> Option<&Ident> {
    path.segments.last().map(|seg| &seg.ident)
}

// Function to transform a block of code
fn transform_function_calls(body: &mut syn::Block) {
    let mut transformer = ProfileCallTransformer;
    transformer.visit_block_mut(body);
}

/// Extracts parameter names from a function signature for forwarding to another function
fn extract_param_names(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
) -> Vec<proc_macro2::TokenStream> {
    inputs
        .iter()
        .map(|arg| match arg {
            // Handle self receiver specially
            syn::FnArg::Receiver(receiver) => {
                if receiver.reference.is_some() {
                    // &self or &mut self
                    if receiver.mutability.is_some() {
                        quote!(self)
                    } else {
                        quote!(self)
                    }
                } else {
                    // self (by value)
                    quote!(self)
                }
            }
            // Handle normal parameters
            syn::FnArg::Typed(pat_type) => {
                match &*pat_type.pat {
                    // Simple identifier (most common case)
                    syn::Pat::Ident(pat_ident) => {
                        let ident = &pat_ident.ident;
                        quote!(#ident)
                    }
                    // Tuple pattern
                    syn::Pat::Tuple(_pat_tuple) => {
                        // For tuple patterns, we need to refer to the whole pattern
                        // This is a simplification; more complex patterns might need special handling
                        let pattern = &*pat_type.pat;
                        quote!(#pattern)
                    }
                    // Struct pattern
                    syn::Pat::Struct(_pat_struct) => {
                        // For struct patterns, similar to tuples
                        let pattern = &*pat_type.pat;
                        quote!(#pattern)
                    }
                    // Array/slice pattern
                    syn::Pat::Slice(_pat_slice) => {
                        let pattern = &*pat_type.pat;
                        quote!(#pattern)
                    }
                    // Reference pattern
                    syn::Pat::Reference(_pat_ref) => {
                        let pattern = &*pat_type.pat;
                        quote!(#pattern)
                    }
                    // Wildcard pattern (_)
                    syn::Pat::Wild(_) => {
                        // Not directly forwardable - would need a temporary variable
                        // This is a simplification
                        quote!(_)
                    }
                    // Other patterns
                    _ => {
                        // Generic fallback for other pattern types
                        // Note: This is a simplification and might not work for all complex patterns
                        let pattern = &*pat_type.pat;
                        quote!(#pattern)
                    }
                }
            }
        })
        .collect()
}
