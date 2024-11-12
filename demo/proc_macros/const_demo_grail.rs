/// Recycled test suite from `https://github.com/redmcg/const_gen_proc_macro`
use const_gen_proc_macro::{
    Expression, Object, ObjectType, Parameter, Path, ProcMacroEnv, Return, ReturnResult,
};
use proc_macro::TokenStream;
use serde::{Deserialize, Serialize};
use serde_json;
use std::str::FromStr;

mod grail {
    pub struct Grail {
        value: Vec<i128>,
    }

    impl Grail {
        pub fn new(json_string: String) -> Self {
            let value = serde_json::from_str(&json_string)
                .expect("Error deserializing JSON string {json_string}");
            Self { value }
        }

        pub fn get(&self) -> i128 {
            self.value[0]
        }

        pub fn add(&mut self, other: i128) -> i128 {
            self.value[0] += other;
            self.value[0]
        }

        pub fn add_four(&mut self, first: i128, second: i128, third: i128, fourth: i128) {
            self.value[0] += first + second + third + fourth
        }

        pub fn sub(&mut self, other: i128) -> i128 {
            self.value[0] -= other;
            self.value[0]
        }
    }
}

use grail::Grail;
fn add(grail: &mut Grail, param: Parameter) -> ReturnResult {
    Ok(Return::Int(if let Parameter::Object(object) = param {
        let Return::Int(value) = object.call("get", &[])? else {
            return Err("received wrong object".to_string());
        };
        grail.add(value)
    } else {
        grail.add(param.try_into()?)
    }))
}

fn add_to(grail: &Grail, other: Object) -> Result<(), String> {
    other.call("add", &[grail.get().into()])?;
    Ok(())
}

pub fn const_demo_grail_impl(tokens: TokenStream) -> TokenStream {
    let mut grail_type = ObjectType::new();
    grail_type.add_method("get", &(&Grail::get as &dyn Fn(&Grail) -> i128));
    grail_type.add_method_mut(
        "add",
        &(&add as &dyn Fn(&mut Grail, Parameter) -> ReturnResult),
    );
    grail_type.add_method(
        "add_to",
        &(&add_to as &dyn Fn(&Grail, Object) -> Result<(), String>),
    );
    grail_type.add_method_mut(
        "add_four",
        &(&Grail::add_four as &dyn Fn(&mut Grail, i128, i128, i128, i128)),
    );
    grail_type.add_method_mut("sub", &(&Grail::sub as &dyn Fn(&mut Grail, i128) -> i128));
    let grail_type = grail_type.seal();

    let mut math_path = Path::new();
    let math_new = &|value: String| -> Result<Object, String> {
        Ok(grail_type.new_instance(Grail::new(value)))
    } as &dyn Fn(String) -> Result<Object, String>;
    math_path.add_function("new", &math_new);

    let mut expr_path = Path::new();
    expr_path.add_function(
        "custom",
        &(&|value: String| -> Result<Expression, String> {
            Expression::try_from(TokenStream::from_str(&value).map_err(|err| err.to_string())?)
        } as &dyn Fn(String) -> Result<Expression, String>),
    );

    let mut string_type = ObjectType::new();
    string_type.add_method(
        "concat",
        &(&|first: &String, second: String| -> String { first.to_owned() + &second }
            as &dyn Fn(&String, String) -> String),
    );
    let string_type = string_type.seal();

    let mut string_path = Path::new();
    let string_new =
        &|first: String| -> Object { string_type.new_instance(first) } as &dyn Fn(String) -> Object;
    string_path.add_function("new", &string_new);

    string_path.add_function(
        "concat",
        &(&|first: String, second: String| -> String { first + &second }
            as &dyn Fn(String, String) -> String),
    );

    let mut env = ProcMacroEnv::new();
    env.add_path("grail", math_path);
    env.add_path("expr", expr_path);
    env.add_path("string", string_path);
    env.process(tokens)
}
