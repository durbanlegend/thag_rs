#![allow(clippy::uninlined_format_args)]

use rs_script::{execute, get_args};
use std::error::Error;

//      TODO:
//       1.  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs.
//       2.  Consider history support for stdin.
//       3.  Paste event in Windows slow or not happening?
//       4.  How to navigate reedline history entry by entry instead of line by line.
//       6.  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
//       7.  More unit and integration tests
//       8.  Cat files before delete.
//       9.  Decide if it's worth passing the wrapped syntax tree to gen_build_run from eval just to avoid
//           re-parsing it for that specific use case.
//      10.  Clean up debugging
//  >>> 11.  Replace dbg by something less funky
//      12.  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
//      13.  Clap aliases not working in REPL.
//      14.  Get rid of date and time in RHS of REPL? - doesn't seem to be an option.
//      15.  Help command in eval, same as quit and q
//      16.  Work on demo/reedline_clap_repl_gemini.rs
//      17.  Put the more intractable long-term problems here in a separate TODO file?
//      18.  How to set editor in Windows.
//      19.  Verbosity enum as per cargo crate?

#[allow(clippy::too_many_lines)]
pub fn main() -> Result<(), Box<dyn Error>> {
    let args = get_args();
    execute(args)?;

    Ok(())
}
