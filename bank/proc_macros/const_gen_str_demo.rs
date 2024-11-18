use const_gen_proc_macro::{Object, ObjectType, Path, ProcMacroEnv};
use proc_macro::TokenStream;

pub fn string_concat_impl(tokens: TokenStream) -> TokenStream {
    let mut string_type = ObjectType::new();
    string_type.add_method(
        "concat",
        &(&|first: &String, second: String| -> String { first.to_owned() + &second }
            as &dyn Fn(&String, String) -> String),
    );
    // sealing the ObjectType means it is no longer mutable and can now instantiate objects
    let string_type = string_type.seal();

    let mut string_path = Path::new();
    let string_new =
        &|first: String| -> Object { string_type.new_instance(first) } as &dyn Fn(String) -> Object;
    string_path.add_function("new", &string_new);

    let mut env = ProcMacroEnv::new();
    env.add_path("string", string_path);
    env.process(tokens)
}
