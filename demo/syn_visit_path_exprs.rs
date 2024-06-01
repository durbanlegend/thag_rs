/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.60", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

use std::{
    env, fs,
    io::{self, Read},
    path::PathBuf,
};

const INPUT_CODE: &str = stringify! {
    fn main() {
        let other = std::string::String::from("world");
        println!("Hello, {other}!");
    }
};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    use ::quote::ToTokens;
    use ::syn::{visit::*, *};

    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => panic!("Couldn't find filepath arg"),
    };
    let content = fs::read_to_string(&filepath).expect("Error reading file");
    let code: File = parse_file(&content).unwrap();

    struct FindCrates;
    impl<'ast> Visit<'ast> for FindCrates {
        fn visit_use_rename(&mut self, node: &'ast syn::ExprPath) {
            println!("Path with first segment={:#?}", node.path.segments.first());
        }
    }

    FindCrates.visit_file(&code);
    // println!("{:#?}", code.into_token_stream());
}
