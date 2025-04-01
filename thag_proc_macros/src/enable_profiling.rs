#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

// #[derive(Debug)]
// enum ProfileTypeOverride {
//     Time,
//     Memory,
//     Both,
// }

// struct ProfileArgs {
//     profile_type: Option<ProfileTypeOverride>,
// }

// // Custom parsing for the attribute arguments
// impl Parse for ProfileArgs {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let profile_type = if input.is_empty() {
//             None
//         } else {
//             let ident: Ident = input.parse()?;
//             if ident != "profile_type" {
//                 return Err(syn::Error::new(ident.span(), "Expected 'type'"));
//             }
//             let _: Token![=] = input.parse()?;
//             let type_str: LitStr = input.parse()?;
//             Some(match type_str.value().as_str() {
//                 "time" => ProfileTypeOverride::Time,
//                 "memory" => ProfileTypeOverride::Memory,
//                 "both" => ProfileTypeOverride::Both,
//                 _ => {
//                     return Err(syn::Error::new(
//                         type_str.span(),
//                         "Invalid profile type. Expected 'time', 'memory', or 'both'",
//                     ))
//                 }
//             })
//         };

//         Ok(Self { profile_type })
//     }
// }

pub fn enable_profiling_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Runtime check for feature flag to handle when the proc macro
    // is compiled with the feature but used without it
    if cfg!(not(feature = "profiling")) {
        // No wrapper, return original function
        return item;
    }

    // let args = parse_macro_input!(attr as ProfileArgs);
    let input = parse_macro_input!(item as ItemFn);

    // Check if the function is async
    let is_async = input.sig.asyncness.is_some();
    eprintln!("is_async={is_async}, input={input:#?}");

    // let profile_type = match args.profile_type {
    //     Some(ProfileTypeOverride::Time) => quote! { ProfileType::Time },
    //     Some(ProfileTypeOverride::Memory) => quote! { ProfileType::Memory },
    //     Some(ProfileTypeOverride::Both) | None => quote! { ProfileType::Both },
    // };

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    let base_location = quote! {
        use backtrace::{Backtrace, BacktraceFrame};

        static_lazy! {
            BaseLocation: &'static str =  Box::leak(Backtrace::frames(&Backtrace::new())
                .iter()
                .flat_map(BacktraceFrame::symbols)
                .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
                .skip_while(|frame| !(frame.contains(module_path!())))
                .take(1)
                .last()
                .unwrap().into_boxed_str())
        }
    };

    let result = if is_async {
        // Handle async function
        quote! {

            #base_location

            #(#attrs)*
            #vis async fn #fn_name #generics(#inputs) #output #where_clause {
                // Initialize profiling
                thag_profiler::init_profiling();

                // For async functions, we need to use an async block
                let result = async {
                    #block
                }.await;

                // Finalize profiling
                thag_profiler::finalize_profiling();
                result
            }
        }
    } else {
        // Handle non-async function (existing implementation)
        quote! {

            #base_location

            #(#attrs)*
            #vis fn #fn_name #generics(#inputs) #output #where_clause {
                // Initialize profiling
                thag_profiler::init_profiling();

                let result = (|| {
                    #block
                })();

                // Finalize profiling
                thag_profiler::finalize_profiling();
                result
            }
        }
    };

    result.into()
}
