#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Receiver};

pub fn profile_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &generics.where_clause;
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

    // Include generic parameters in profile name
    let type_params: Vec<_> = generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(t) => t.ident.to_string(),
            syn::GenericParam::Lifetime(l) => l.lifetime.to_string(),
            syn::GenericParam::Const(c) => c.ident.to_string(),
        })
        .collect();

    // let type_str =  if let Some(impl_block) = get_impl_block(&input) {
    //         format!("{}::{}", impl_block.self_ty, fn_name)
    //     } else {
    //         format!("method::{}", fn_name)
    //     }
    // }
    let profile_name = match (is_method, type_params.is_empty()) {
        (true, true) => format!("method::{}", fn_name),
        (true, false) => format!(
            "{}::{fn_name}<{}>",
            impl_block.self_ty,
            type_params.join(",")
        ),
        (false, true) => todo!(),
        (false, false) => format!("fn::{}", fn_name),
        //     if let Some(impl_block) = get_impl_block(&input) {
        //         format!("{}::{}", impl_block.self_ty, fn_name)
        //     } else {
        //         format!("method::{}", fn_name)
        //     }
        // }) else {
        //     format!("fn::{}", fn_name)
    };

    let profile_name = if type_params.is_empty() {
        format!("fn::{}", fn_name)
    } else {
        format!("fn::{}<{}>", fn_name, type_params.join(","))
    };

    let wrapped = quote! {
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
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
