/*[toml]
[dependencies]
edit = { version = "0.1.5", features = ["quoted-env", "shell-words"] }
*/

#[allow(unused_doc_comments)]
/// Published example from edit crate readme.
///
/// Will use the editor specified in VISUAL or EDITOR environment variable.
///
/// E.g. `EDITOR=vim rs_script demo/edit.rs`
//# Purpose: Demo of edit crate to invoke preferred editor.
let template = "Fill in the blank: Hello, _____!";
let edited = edit::edit(template)?;
println!("after editing: '{}'", edited);
// after editing: 'Fill in the blank: Hello, world!'
