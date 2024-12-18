/*[toml]
[dependencies]
thag_proc_macros = { path = "/Users/donf/projects/thag_rs/thag_proc_macros" }
*/
use thag_proc_macros::MyDescription;

#[derive(MyDescription, Default)]
#[my_desc(name = "hello world", version = "0.2")]
struct Hello {
    a: i32,
    b: String
}

let hello: Hello = Default::default();
assert_eq!(hello.my_desc(), "Name: hello world, Version: 0.2");

println!("{}", hello.my_desc());
