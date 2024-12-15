/*[toml]
[dependencies]
tempfile = "3.10.1"
*/
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

/// Published example from the `tempfile` readme.
//# Purpose: Demo featured crate.
//# Categories: crates
fn main() {
    // Write
    let mut tmpfile: File = tempfile::tempfile().unwrap();
    write!(tmpfile, "Hello World!").unwrap();

    // Seek to start
    tmpfile.seek(SeekFrom::Start(0)).unwrap();

    // Read
    let mut buf = String::new();
    tmpfile.read_to_string(&mut buf).unwrap();
    assert_eq!("Hello World!", buf);
}
