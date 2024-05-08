/*[toml]
[dependencies]
nu-ansi-term = { version = "0.50.0", features = ["derive_serde_style"] }
*/

use nu_ansi_term::{Color, Style};
use std::io::{self, BufRead, Read};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_line(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    let outer_prompt = || {
        println!(
            "{}",
            nu_ansi_term::Color::Magenta
                .bold()
                .paint(format!("Enter one of: {}", "one, two, three"))
        );
    };
    outer_prompt();
    // let mut loop_editor = ClapEditor::<LoopCommand>::new();
    // let mut loop_command = loop_editor.read_command();
    'level2: loop {
        loop {
            let style = Style::new().bold().fg(Color::Cyan);
            println!(
                "{}",
                style.paint(
                    r"Enter an expression (e.g., 2 + 3), or q to quit.
        Expressions in matching braces, brackets or quotes may span multiple lines."
                )
            );

            let content = read_stdin().expect("Problem reading input");
            println!("content={content}");
        }
    }
}
