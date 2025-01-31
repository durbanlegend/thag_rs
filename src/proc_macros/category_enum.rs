#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;

pub fn category_enum_impl(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
       // use crate::lazy_static_var;
       use std::str::FromStr;
       use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

       #[derive(Debug, Clone, Copy, Display, PartialEq, Eq, EnumIter, EnumString, IntoStaticStr)]
       #[strum(serialize_all = "snake_case", use_phf)]
       enum Category {
           AST,
           CLI,
           REPL,
           Async,
           Basic,
           BigNumbers,
           Crates,
           Demo,
           Educational,
           ErrorHandling,
           Exploration,
           Filesystem,
           Macros,
           Math,
           ProcMacros,
           Profiling,
           Prototype,
           Recreational,
           Reference,
           Technique,
           Testing,
           ThagFrontEnds,
           Tools,
           TypeIdentification,
       }

       /// Returns a vector of all valid category names as strings.
       ///
       /// This function is automatically generated by the `category_enum` macro and provides
       /// a complete list of categories, making it convenient for validation, UI prompts, or filtering.
       ///
       /// # Example
       ///
       /// ```rust
       /// use demo_proc_macros::category_enum;
       ///
       /// category_enum! {}
       ///
       /// let categories = Category::all_categories();
       /// assert_eq!(categories, vec![
       ///     "ast", "cli", "repl", "async", "basic", "big_numbers", "crates", "demo",
       ///     "educational", "error_handling", "exploration", "filesystem", "macros", "math",
       ///     "proc_macros", "profiling", "prototype", "recreational", "reference", "technique",
       ///     "testing", "thag_front_ends", "tools", "type_identification"
       /// ]);
       /// ```
        pub fn all_categories() -> Vec<String> {
            let v = lazy_static_var!(Vec<String>, {
                Category::iter()
                    .map(|variant| variant.to_string())
                    .collect::<Vec<String>>()
            });
            v.clone()
        }
    };
    TokenStream::from(expanded)
}
