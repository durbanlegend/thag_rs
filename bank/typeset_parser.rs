/*[toml]
[dependencies]
typeset = "2.0.4"
*/
// use typeset;
use typeset_parser::layout;

fn main() {
    let foo = typeset::text("foo".to_string());
    let foobar = layout! {
      fix (nest (foo & "bar")) @
      pack ("baz" !+ foo) @@
      grp null + seq (foo + foo !& foo)
    };
    let document = typeset::compile(foobar.clone());
    println!("---------------------");
    println!("{}", foobar);
    println!("---------------------");
    println!("{}", document);
    println!("---------------------");
    println!("\"{}\"", typeset::render(document, 2, 80));
    println!("---------------------");
}
