use std::io::{self, Write, Read};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() -> io::Result<()> {
    enable_raw_mode()?; // disable canonical/echo
    let mut stdout = io::stdout();
    write!(stdout, "\x1b]11;?\x07")?;
    stdout.flush()?;

    // Read the response directly (blocking)
    let mut buf = [0u8; 64];
    let _n = io::stdin().read(&mut buf)?;
    // println!("Response: {:?}", String::from_utf8_lossy(&buf[..n]));

    disable_raw_mode()?;
    Ok(())
}
