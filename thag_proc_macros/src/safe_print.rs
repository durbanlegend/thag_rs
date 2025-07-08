//! Safe print macros for terminal synchronization
//!
//! This module provides synchronized print macros that prevent terminal corruption
//! from concurrent OSC sequences or thread interference during testing.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, Token};

/// Input for the safe print macros
struct SafePrintInput {
    format_str: Expr,
    args: Vec<Expr>,
}

impl Parse for SafePrintInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let format_str = input.parse()?;
        let mut args = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            args.push(input.parse()?);
        }

        Ok(SafePrintInput { format_str, args })
    }
}

/// Implementation for safe_print! macro
pub fn safe_print_impl(input: TokenStream) -> TokenStream {
    let SafePrintInput { format_str, args } = parse_macro_input!(input as SafePrintInput);

    let output = if args.is_empty() {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                print!(#format_str);
                let _ = stdout().flush();
            }
        }
    } else {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                print!(#format_str, #(#args),*);
                let _ = stdout().flush();
            }
        }
    };

    output.into()
}

/// Implementation for safe_println! macro
pub fn safe_println_impl(input: TokenStream) -> TokenStream {
    let SafePrintInput { format_str, args } = parse_macro_input!(input as SafePrintInput);

    let output = if args.is_empty() {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                println!(#format_str);
                let _ = stdout().flush();
            }
        }
    } else {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                println!(#format_str, #(#args),*);
                let _ = stdout().flush();
            }
        }
    };

    output.into()
}

/// Implementation for safe_eprint! macro
pub fn safe_eprint_impl(input: TokenStream) -> TokenStream {
    let SafePrintInput { format_str, args } = parse_macro_input!(input as SafePrintInput);

    let output = if args.is_empty() {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stderr, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                eprint!(#format_str);
                let _ = stderr().flush();
            }
        }
    } else {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stderr, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                eprint!(#format_str, #(#args),*);
                let _ = stderr().flush();
            }
        }
    };

    output.into()
}

/// Implementation for safe_eprintln! macro
pub fn safe_eprintln_impl(input: TokenStream) -> TokenStream {
    let SafePrintInput { format_str, args } = parse_macro_input!(input as SafePrintInput);

    let output = if args.is_empty() {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stderr, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                eprintln!(#format_str);
                let _ = stderr().flush();
            }
        }
    } else {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stderr, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                eprintln!(#format_str, #(#args),*);
                let _ = stderr().flush();
            }
        }
    };

    output.into()
}

/// Implementation for safe_osc! macro - for OSC sequences
pub fn safe_osc_impl(input: TokenStream) -> TokenStream {
    let SafePrintInput { format_str, args } = parse_macro_input!(input as SafePrintInput);

    let output = if args.is_empty() {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                print!(#format_str);
                let _ = stdout().flush();
            }
        }
    } else {
        quote! {
            {
                use std::sync::LazyLock;
                use std::sync::Mutex;
                use std::io::{stdout, Write};

                static TERMINAL_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

                let _guard = TERMINAL_MUTEX.lock().unwrap();
                print!(#format_str, #(#args),*);
                let _ = stdout().flush();
            }
        }
    };

    output.into()
}
