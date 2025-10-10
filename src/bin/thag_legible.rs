/*[toml]
[dependencies]
thag_common = { version = "0.2, thag-auto" }

[profile.dev]
opt-level = 2
*/

/// Unescape \n and \\" markers in a string to convert the wall of text to readable lines.
/// This is trickier than it seems because in a compile-time literal, \n compiles to the
/// true line feed character 10 (x0A), whereas a \n generated or captured as a literal
/// at run time is encoded as ('\', 'n'() = (92, 110) = 0x5c6e. Not surprisingly, the two
/// representations, while they look identical to the programmer, don't always behave
/// the same.
///
/// See `demo/dethagomizer.rs` for a Regex version.
//# Purpose: Useful script for converting a wall of text such as some TOML errors back into legible formatted messages.
//# Categories: crates, technique, tools
use std::io::{self, Read};
use thag_common::{auto_help, help_system::check_help_and_exit, set_verbosity_from_env, vprtln, V};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn reassemble<'a>(iter: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    iter.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

// Unescape \n markers in a string to convert the wall of text to readable lines.
fn dethagomize(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

fn main() {
    // Check for help first - automatically extracts from source comments
    let help = auto_help!();
    check_help_and_exit(&help);

    set_verbosity_from_env();

    vprtln!(
        V::N,
        "Type text wall at the prompt and hit Ctrl-D on a new line when done"
    );

    // Remove backslash escapes from double quotes.
    let esc_dq = r#"\""#;
    let dq = r#"""#;

    let content = read_stdin()
        .expect("Problem reading input")
        .replace("\\n", "\n") // Have to replace because raw data strings are treated differently from hard-coded strings
        .replace(esc_dq, dq);
    vprtln!(V::N, "\nDethagomized:");
    println!("{}", dethagomize(&content));
    // vprtln!(V::N);
}
