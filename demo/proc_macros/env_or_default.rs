#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr, Token};

/// Parse the input for `env_or_default!("ENV_VAR`", "`default_value`")
struct EnvOrDefaultInput {
    env_var: LitStr,
    default_value: LitStr,
}

impl Parse for EnvOrDefaultInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let env_var = input.parse()?;
        input.parse::<Token![,]>()?;
        let default_value = input.parse()?;
        Ok(EnvOrDefaultInput {
            env_var,
            default_value,
        })
    }
}

pub fn env_or_default_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as EnvOrDefaultInput);
    let env_var_name = input.env_var.value();

    // Try to get the environment variable at compile time
    let env_value = std::env::var(&env_var_name).unwrap_or_else(|_| input.default_value.value());

    // Generate a string literal with the resolved value
    let expanded = quote! {
        #env_value
    };

    TokenStream::from(expanded)
}
