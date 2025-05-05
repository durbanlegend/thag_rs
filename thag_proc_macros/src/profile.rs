#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, Result, Token,
};

/// Arguments for the `profile` macro
struct ProfileArgs {
    name: String,
    args: Punctuated<Ident, Token![,]>,
}

impl Parse for ProfileArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the profile name as an identifier
        let name = input.parse::<Ident>()?.to_string();

        // Parse remaining arguments as identifiers separated by commas
        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![,]>()?;
            input.parse_terminated(Ident::parse, Token![,])?
        };

        Ok(Self { name, args })
    }
}

pub fn profile_impl(input: TokenStream) -> TokenStream {
    let ProfileArgs { name, args } = parse_macro_input!(input as ProfileArgs);

    // Extract flags from args
    let has_time = args.iter().any(|arg| arg == "time" || arg == "both");
    let has_mem_summary = args.iter().any(|arg| arg == "mem_summary" || arg == "both");
    let has_mem_detail = args.iter().any(|arg| arg == "mem_detail");
    let is_async = args.iter().any(|arg| arg == "async_fn");
    let is_unbounded = args.iter().any(|arg| arg == "unbounded");
    let use_global = args.iter().any(|arg| arg == "global");

    // Determine profile type
    let profile_type = if use_global {
        quote! { ::thag_profiler::get_global_profile_type() }
    } else if has_time && (has_mem_summary || has_mem_detail) {
        quote! { ::thag_profiler::ProfileType::Both }
    } else if has_time {
        quote! { ::thag_profiler::ProfileType::Time }
    } else {
        quote! { ::thag_profiler::get_global_profile_type() }
    };

    // Determine if detailed memory is enabled
    let detailed_memory = has_mem_detail;

    // Determine line numbers
    let (start_line, end_line) = if has_time && !(has_mem_summary || has_mem_detail) {
        // Time only - no line numbers needed
        (quote! { None }, quote! { None })
    } else if is_unbounded {
        // Memory with unbounded - only start line
        (quote! { Some(line!()) }, quote! { None })
    } else {
        // Memory with bounded - need end marker
        // Convert the string to an identifier
        let end_line_fn = format_ident!("end_{name}");
        (quote! { Some(line!()) }, quote! { Some(#end_line_fn()) })
    };

    let profile_id = format_ident!("{name}");

    // Generate the profile creation code
    #[cfg(not(feature = "full_profiling"))]
    let expanded = quote! {
        let #profile_id = ::thag_profiler::Profile::new(
            Some(#name),
            None,
            #profile_type,
            #is_async,
            #detailed_memory,
            file!(),
            #start_line,
            #end_line
        );
    };

    #[cfg(feature = "full_profiling")]
    let expanded = quote! {
        let #profile_id = ::thag_profiler::with_allocator(::thag_profiler::Allocator::System, || {
            ::thag_profiler::Profile::new(
                Some(#name),
                None,
                #profile_type,
                #is_async,
                #detailed_memory,
                file!(),
                #start_line,
                #end_line
            )
        });
    };

    expanded.into()
}