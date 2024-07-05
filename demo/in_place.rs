/*[toml]
[dependencies]
in-place = "0.2.0"
*/

use in_place::InPlace;
use std::io::{BufRead, BufReader, Write};

/// Published example from `in-place crate` disemvowels the file somefile.txt.
//# Purpose: Demo editing a file in place.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inp = InPlace::new("somefile.txt")
        .backup(in_place::Backup::Append("~".into()))
        .open()?;
    let reader = BufReader::new(inp.reader());
    let mut writer = inp.writer();
    for line in reader.lines() {
        let mut line = line?;
        line.retain(|ch| !"AEIOUaeiou".contains(ch));
        writeln!(writer, "{line}")?;
    }
    inp.save()?;
    Ok(())
}
