#![allow(clippy::missing_panics_doc, unused_imports)]
mod attrib_key_map_list;
mod const_gen_demo;
mod custom_model;
mod derive_deserialize_vec;
mod derive_key_map_list;
mod expander_demo;
mod into_string_hash_map;
mod my_description;
mod organizing_code;
mod organizing_code_const;
mod organizing_code_tokenstream;

use crate::attrib_key_map_list::use_mappings_impl;
use crate::const_gen_demo::string_concat_impl;
use crate::custom_model::derive_custom_model_impl;
use crate::derive_deserialize_vec::derive_deserialize_vec_impl;
use crate::derive_key_map_list::derive_key_map_list_impl;
use crate::expander_demo::baz2;
use crate::into_string_hash_map::into_hash_map_impl;
use crate::my_description::my_derive;
use crate::organizing_code::organizing_code_impl;
use crate::organizing_code_const::organizing_code_const_impl;
use crate::organizing_code_tokenstream::organizing_code_tokenstream_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ExprArray, Ident, LitInt, Token,
};

#[proc_macro_derive(DeriveCustomModel, attributes(custom_model))]
pub fn derive_custom_model(item: TokenStream) -> TokenStream {
    derive_custom_model_impl(item)
}

#[proc_macro_derive(IntoStringHashMap)]
pub fn into_hash_map(item: TokenStream) -> TokenStream {
    into_hash_map_impl(item)
}

#[proc_macro_derive(MyDescription, attributes(my_desc))]
pub fn derive_my_description(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    my_derive(item.into()).unwrap().into()
}

// Define the custom derive macro using `deluxe`
#[proc_macro_derive(DeserializeVec, attributes(deluxe, use_mappings))]
pub fn derive_deserialize_vec(input: TokenStream) -> TokenStream {
    derive_deserialize_vec_impl(input.into()).unwrap().into()
}

#[proc_macro_derive(DeriveKeyMapList, attributes(deluxe, use_mappings))]
pub fn derive_key_map_list(item: TokenStream) -> TokenStream {
    derive_key_map_list_impl(item.into()).unwrap().into()
}

// From https://github.com/tdimitrov/rust-proc-macro-post
#[proc_macro]
pub fn organizing_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_impl(input).into()
}

// From https://github.com/tdimitrov/rust-proc-macro-post
#[proc_macro]
pub fn organizing_code_tokenstream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    organizing_code_tokenstream_impl(input).into()
}

#[proc_macro_derive(DeriveConst, attributes(adjust, use_mappings))]
pub fn organizing_code_const(input: TokenStream) -> TokenStream {
    organizing_code_const_impl(input.into()).unwrap().into()
}

#[proc_macro_attribute]
pub fn baz(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // wrap as per usual for `proc-macro2::TokenStream`, here dropping `attr` for simplicity
    baz2(input.into()).into()
}

#[proc_macro_attribute]
pub fn use_mappings(attr: TokenStream, item: TokenStream) -> TokenStream {
    use_mappings_impl(attr, item)
}

#[proc_macro]
pub fn repeat_dash(input: TokenStream) -> TokenStream {
    // Parse the input as a literal integer
    let input = parse_macro_input!(input as LitInt);
    let len = input
        .base10_parse::<usize>()
        .expect("Expected a usize integer");

    // Generate the repeated dash string
    let dash_line = "-".repeat(len);

    // expanded a constant string definition
    TokenStream::from(quote! {
        const DASH_LINE: &str = #dash_line;
    })
}

#[proc_macro]
pub fn string_concat(tokens: TokenStream) -> TokenStream {
    string_concat_impl(tokens)
}

use const_gen_proc_macro::{
    Expression, Object, ObjectType, Parameter, Path, ProcMacroEnv, Return, ReturnResult,
};
use std::str::FromStr;

mod string_array {
    pub struct StringArray {
        values: Vec<String>,
    }

    impl StringArray {
        pub fn new(values: Vec<String>) -> Self {
            Self { values }
        }

        pub fn merge(&mut self, other: Vec<String>) {
            self.values.extend(other);
        }

        pub fn get_values(&self) -> Vec<String> {
            self.values.clone()
        }
    }
}

use string_array::StringArray;
// Updated string_array_new function
fn string_array_new(param: Parameter) -> Result<Object, String> {
    // Check if the parameter is an array of strings
    if let Parameter::Array(boxed_array) = param {
        let values: Vec<String> = boxed_array
            .iter()
            .map(|item| match item {
                Parameter::String(s) => Ok(s.clone()),
                _ => Err("Expected an array of strings".to_string()),
            })
            .collect::<Result<_, _>>()?; // Collect results, propagating any errors

        // Create a new instance of StringArray if successful
        let string_array = StringArray::new(values);
        let string_array_type = ObjectType::new(); // Assuming ObjectType is already sealed elsewhere
        let string_array_type = string_array_type.seal();

        Ok(string_array_type.new_instance(string_array))
    } else {
        Err("Expected Parameter::Array(Box<[Parameter::String]>)".to_string())
    }
}

fn merge_arrays(array: &mut StringArray, param: Parameter) -> ReturnResult {
    if let Parameter::Object(object) = param {
        if let Return::Array(arr) = object.call("get_values", &[])? {
            let strings: Vec<String> = arr
                .iter()
                .filter_map(|ret| {
                    if let Return::String(s) = ret {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            array.merge(strings);
            Ok(Return::Object(object.clone())) // Return an updated object
        } else {
            Err("Expected an Array of Strings.".to_string())
        }
    } else {
        Err("Expected an Object parameter.".to_string())
    }
}

fn get_array(array: &StringArray) -> Return {
    let values = array
        .get_values()
        .into_iter()
        .map(Return::String)
        .collect::<Vec<_>>();
    Return::Array(values.into_boxed_slice())
}

// Updated proc_macro to use string_array_new with a single Parameter argument
#[proc_macro]
pub fn string_array_macro(tokens: TokenStream) -> TokenStream {
    let mut string_array_type = ObjectType::new();

    string_array_type.add_method(
        "get_values",
        &(&get_array as &dyn Fn(&StringArray) -> Return),
    );
    string_array_type.add_method_mut(
        "merge",
        &(&merge_arrays as &dyn Fn(&mut StringArray, Parameter) -> ReturnResult),
    );

    let mut string_array_path = Path::new();
    string_array_path.add_function(
        "new",
        &(&string_array_new as &dyn Fn(Parameter) -> Result<Object, String>),
    );

    let mut env = ProcMacroEnv::new();
    env.add_path("string_array", string_array_path);
    env.process(tokens)
}
