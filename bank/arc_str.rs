use std::sync::Arc;
println!("{}", Arc::<str>::from(r#"Arc::<str>::from("Hello world!")"#));

let arc_str = <Arc<str> as From<&str>>::from(r#"<Arc<str> as From<&str>>::from("hello world")"#);
println!("{arc_str}");
