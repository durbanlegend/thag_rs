/*[toml]
[dependencies]
syn = {version = "2.0.82", features = ["extra-traits", "full", "parsing"] }
*/

/// Tries to convert input to a `syn` abstract syntax tree.
//# Purpose: Debugging
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::path::Path;

use syn;

// fn render_location(
//     formatter: &mut fmt::Formatter,
//     err: &syn::Error,
//     code: &str,
// ) -> fmt::Result {
//     let start = err.span().start();
//     let mut end = err.span().end();

//     let code_line = match start.line.checked_sub(1).and_then(|n| code.lines().nth(n)) {
//         Some(line) => line,
//         None => return render_fallback(formatter, err),
//     };

//     if end.line > start.line {
//         end.line = start.line;
//         end.column = code_line.len();
//     }

//     write!(
//         formatter,
//         "\n\
//          {error}{header}\n\
//          {indent} {pipe}\n\
//          {label} {pipe} {code}\n\
//          {indent} {pipe} {offset}{underline} {message}\n\
//          ",
//         error = "error".red().bold(),
//         header = ": Syn unable to parse file".bold(),
//         indent = " ".repeat(start.line.to_string().len()),
//         arrow = "-->".blue().bold(),
//         linenum = start.line,
//         colnum = start.column,
//         pipe = "|".blue().bold(),
//         label = start.line.to_string().blue().bold(),
//         code = code_line.trim_end(),
//         offset = " ".repeat(start.column),
//         underline = "^"
//             .repeat(end.column.saturating_sub(start.column).max(1))
//             .red()
//             .bold(),
//         message = err.to_string().red(),
//     )
// }

// fn render_fallback(formatter: &mut fmt::Formatter, err: &syn::Error) -> fmt::Result {
//     write!(formatter, "Unable to parse file: {}", err)
// }

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

let content = read_stdin().expect("Problem reading input");
println!("[{:#?}]", content);
let syntax: syn::File = syn::parse_str(&content)?;
println!("{:#?}", syntax);
