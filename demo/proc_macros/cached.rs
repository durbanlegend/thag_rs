#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

pub fn cached_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    // Generate cache name based on function name
    let cache_name = quote::format_ident!("__{}_CACHE", fn_name.to_string().to_uppercase());

    // Extract return type for cache
    let return_type = match output {
        ReturnType::Default => {
            return syn::Error::new_spanned(
                fn_name,
                "Cached functions must have an explicit return type",
            )
            .to_compile_error()
            .into();
        }
        ReturnType::Type(_, ty) => ty,
    };

    // Create cache key type - we'll use a tuple of all input parameters
    let param_types: Vec<_> = inputs
        .iter()
        .filter_map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                Some(&pat_type.ty)
            } else {
                None // Skip 'self' parameters
            }
        })
        .collect();

    let param_names: Vec<_> = inputs
        .iter()
        .filter_map(|param| {
            if let syn::FnArg::Typed(pat_type) = param {
                Some(&pat_type.pat)
            } else {
                None
            }
        })
        .collect();

    // Generate cache key tuple
    let cache_key_type = if param_types.len() == 1 {
        // Single parameter - use it directly
        quote! { #(#param_types)* }
    } else if param_types.is_empty() {
        // No parameters - use unit type
        quote! { () }
    } else {
        // Multiple parameters - use tuple
        quote! { (#(#param_types,)*) }
    };

    let cache_key_value = if param_names.len() == 1 {
        // Single parameter
        quote! { #(#param_names)*.clone() }
    } else if param_names.is_empty() {
        // No parameters
        quote! { () }
    } else {
        // Multiple parameters - create tuple
        quote! { (#(#param_names.clone(),)*) }
    };

    // Check if all parameter types implement Clone
    let clone_bounds = param_types.iter().map(|ty| {
        quote! { #ty: Clone }
    });

    let result = quote! {
        // Static cache using lazy_static pattern with std::sync::Mutex
        static #cache_name: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<#cache_key_type, #return_type>>> =
            std::sync::OnceLock::new();

        #(#attrs)*
        #vis fn #fn_name #generics(#inputs) #output
        #where_clause
        where
            #return_type: Clone,
            #(#clone_bounds,)*
        {
            // Get or initialize the cache
            let cache = #cache_name.get_or_init(|| {
                std::sync::Mutex::new(std::collections::HashMap::new())
            });

            // Create cache key
            let key = #cache_key_value;

            // Check if value is already cached
            {
                let cache_guard = cache.lock().unwrap();
                if let Some(cached_value) = cache_guard.get(&key) {
                    return cached_value.clone();
                }
            }

            // Call original function
            let result = {
                #block
            };

            // Store result in cache
            {
                let mut cache_guard = cache.lock().unwrap();
                cache_guard.insert(key, result.clone());
            }

            result
        }
    };

    result.into()
}
