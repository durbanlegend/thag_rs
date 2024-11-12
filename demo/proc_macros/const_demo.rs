//! This crate is used to test the const_gen_proc_macro library
use const_gen_proc_macro::{
    Expression, Object, ObjectType, Parameter, Path, ProcMacroEnv, Return, ReturnResult,
};
use proc_macro::TokenStream;
use std::str::FromStr;

mod math {
    pub struct Math {
        value: i128,
    }

    impl Math {
        pub fn new(value: i128) -> Self {
            Self { value }
        }

        pub fn get(&self) -> i128 {
            self.value
        }

        pub fn add(&mut self, other: i128) -> i128 {
            self.value += other;
            self.value
        }

        pub fn add_four(&mut self, first: i128, second: i128, third: i128, fourth: i128) {
            self.value += first + second + third + fourth
        }

        pub fn sub(&mut self, other: i128) -> i128 {
            self.value -= other;
            self.value
        }
    }
}

use math::Math;
fn add(math: &mut Math, param: Parameter) -> ReturnResult {
    Ok(Return::Int(if let Parameter::Object(object) = param {
        let Return::Int(value) = object.call("get", &[])? else {
            return Err("received wrong object".to_string());
        };
        math.add(value)
    } else {
        math.add(param.try_into()?)
    }))
}

fn add_to(math: &Math, other: Object) -> Result<(), String> {
    other.call("add", &[math.get().into()])?;
    Ok(())
}

pub fn const_demo_impl(tokens: TokenStream) -> TokenStream {
    let mut math_type = ObjectType::new();
    math_type.add_method("get", &(&Math::get as &dyn Fn(&Math) -> i128));
    math_type.add_method_mut(
        "add",
        &(&add as &dyn Fn(&mut Math, Parameter) -> ReturnResult),
    );
    math_type.add_method(
        "add_to",
        &(&add_to as &dyn Fn(&Math, Object) -> Result<(), String>),
    );
    math_type.add_method_mut(
        "add_four",
        &(&Math::add_four as &dyn Fn(&mut Math, i128, i128, i128, i128)),
    );
    math_type.add_method_mut("sub", &(&Math::sub as &dyn Fn(&mut Math, i128) -> i128));
    let math_type = math_type.seal();

    let mut math_path = Path::new();
    let math_new =
        &|value: i128| -> Result<Object, String> { Ok(math_type.new_instance(Math::new(value))) }
            as &dyn Fn(i128) -> Result<Object, String>;
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
    env.add_path("math", math_path);
    env.add_path("expr", expr_path);
    env.add_path("string", string_path);
    env.process(tokens)
}
