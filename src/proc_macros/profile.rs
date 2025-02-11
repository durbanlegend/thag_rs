#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Receiver};

pub fn profile_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Extract function name and visibility
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let body = &input.block;

    // Determine if this is a method by checking for self parameter
    let is_method = inputs.iter().any(|arg| {
        if let FnArg::Receiver(Receiver {
            reference: Some(_), ..
        }) = arg
        {
            true
        } else {
            false
        }
    });

    // Generate the profile name
    let profile_name = if is_method {
        // For methods, include the struct/trait name if we can get it
        // This is a simplified version - might want to add more context
        format!("method::{}", fn_name)
    } else {
        format!("fn::{}", fn_name)
    };

    // Generate the wrapped function
    let wrapped = quote! {
        #vis fn #fn_name(#inputs) #output {
            let _profile = ::thag::Profile::new(#profile_name, ::thag::ProfileType::Time);
            #body
        }
    };

    wrapped.into()
}

pub fn profile_async_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if !input.sig.asyncness.is_some() {
        return TokenStream::from(quote! {
            compile_error!("profile_async can only be used with async functions");
        });
    }

    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let body = &input.block;

    let profile_name = format!("async::{}", fn_name);

    // Create a future that's bound to the thread where it starts
    let wrapped = quote! {
        #vis async fn #fn_name(#inputs) #output {
            use std::future::Future;
            use std::pin::Pin;
            use std::task::{Context, Poll};

            struct ProfiledFuture<F> {
                inner: F,
                _profile: Option<::thag::Profile>,
            }

            impl<F: Future> Future for ProfiledFuture<F> {
                type Output = F::Output;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    // Safety: we're not moving any pinned data
                    let this = unsafe { self.as_mut().get_unchecked_mut() };
                    let result = unsafe { Pin::new_unchecked(&mut this.inner) }.poll(cx);
                    if result.is_ready() {
                        // Drop the profile when the future completes
                        this._profile.take();
                    }
                    result
                }
            }

            let future = async #body;
            ProfiledFuture {
                inner: future,
                _profile: Some(::thag::Profile::new(#profile_name, ::thag::ProfileType::Time)),
            }.await
        }
    };

    wrapped.into()
}
