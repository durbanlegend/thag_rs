/// Simple HTTPS GET
///
/// This example is a Rust adaptation of the [C example of the same
/// name](https://curl.se/libcurl/c/https.html).
/// On Linux you may need to install `pkg-config` and `libssl-dev`.
//# Purpose: Demo `curl` implementation.
//# Categories: crates, technique
use curl::easy::Easy;
use std::io::{stdout, Write};

fn main() -> Result<(), curl::Error> {
    let mut curl = Easy::new();

    curl.url("https://raw.githubusercontent.com/durbanlegend/thag_rs/master/demo/hello.rs")?;
    curl.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    curl.perform()?;

    Ok(())
}
