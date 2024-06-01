//! [dependencies]
//! edit = { version = "0.1.5", features = ["quoted-env", "shell-words"] }

let template = "Fill in the blank: Hello, _____!";
let edited = edit::edit(template)?;
println!("after editing: '{}'", edited);
// after editing: 'Fill in the blank: Hello, world!'
