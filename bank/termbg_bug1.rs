@ -1,173 +0,0 @@
/*[toml]
[dependencies]
async-std = "1.12.0"
crossterm = "0.27.0"
is-terminal = "0.4.12"
syn = "2.0.72"
termbg = "0.5.0"
thiserror = "1.0.61"
*/

use crossterm::terminal;
use is_terminal::IsTerminal;
use std::env;
use std::io::{self, Read, Write};
use std::time::Duration;
use thiserror::Error;

// Terminal
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Terminal {
    Screen,
    Tmux,
    XtermCompatible,
    Windows,
    Emacs,
}

// Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("parse error")]
    Parse(String),
    #[error("unsupported")]
    Unsupported,
}

// get detected terminal
#[cfg(not(target_os = "windows"))]
pub fn terminal() -> Terminal {
    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    if env::var("TMUX").is_ok() {
        Terminal::Tmux
    } else {
        let is_screen = if let Ok(term) = env::var("TERM") {
            term.starts_with("screen")
        } else {
            false
        };
        if is_screen {
            Terminal::Screen
        } else {
            Terminal::XtermCompatible
        }
    }
}

// get detected terminal
#[cfg(target_os = "windows")]
pub fn terminal() -> Terminal {
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        if term_program == "vscode" {
            return Terminal::XtermCompatible;
        }
    }

    if env::var("INSIDE_EMACS").is_ok() {
        return Terminal::Emacs;
    }

    // Windows Terminal is Xterm-compatible
    // https://github.com/microsoft/terminal/issues/3718
    if env::var("WT_SESSION").is_ok() {
        Terminal::XtermCompatible
    } else {
        Terminal::Windows
    }
}

fn from_xterm(term: Terminal, timeout: Duration) -> Result<(), Error> {
    if !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
        || !std::io::stderr().is_terminal()
    {
        // Not a terminal, so don't try to read the current background color.
        return Err(Error::Unsupported);
    }

    // Query by XTerm control sequence
    println!("term={term:#?}");
    let query = if term == Terminal::Tmux {
        "\x1bPtmux;\x1b\x1b]11;?\x07\x1b\\\x03"
    } else if term == Terminal::Screen {
        "\x1bP\x1b]11;?\x07\x1b\\\x03"
    } else {
        "\x1b]11;?\x1b\\"
    };

    let mut stderr = io::stderr();
    terminal::enable_raw_mode()?;
    write!(stderr, "{}", query)?;
    stderr.flush()?;

    let buffer = async_std::task::block_on(async_std::io::timeout(timeout, async {
        use async_std::io::ReadExt;
        // let mut buffer = Vec::new();
        let mut buffer = String::new();
        let mut stdin = async_std::io::stdin();
        let mut buf = [0; 25];
        println!("buf.len()={}", buf.len());
        let mut start = false;
        let _ = stdin.read(&mut buf).await?;
        // loop {
        //     let _ = stdin.read_exact(&mut buf).await?;
        //     print!("{:#?},", char::from(buf[0]));
        //     // response terminated by BEL(0x7)
        //     if start && (buf[0] == 0x7) {
        //         continue;
        //     }
        //     // response terminated by ST(0x1b 0x5c)
        //     if start && (buf[0] == 0x1b) {
        //         // consume last 0x5c
        //         let _ = stdin.read_exact(&mut buf).await?;
        //         debug_assert_eq!(buf[0], 0x5c);
        //         continue;
        //     }
        //     if start {
        //         buffer.push(buf[0] as char);
        //     }
        //     if buf[0] == b':' {
        //         start = true;
        //     }
        // }
        Ok(buf)
    }));

    terminal::disable_raw_mode()?;

    if let Ok(buf) = buffer {
        println!("s={:#?}", String::from_utf8(buf.to_vec()));
    } else {
        println!("Err={buffer:#?}");
    }

    // // Should return by error after disable_raw_mode
    // let buffer = buffer?;

    // let s = String::from_utf8_lossy(&buffer);
    // println!("s={s:#?}");

    Ok(())
}

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    // let rgb = termbg::rgb(timeout);
    // // let theme = termbg::theme(timeout);
    let term = terminal();
    from_xterm(term, timeout).unwrap();

    println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");
}
