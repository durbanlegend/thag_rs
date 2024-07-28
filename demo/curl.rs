/*[toml]
[dependencies]
curl = "0.4.46"
*/
/// Simple HTTPS GET
///
/// This example is a Rust adaptation of the [C example of the same
/// name](https://curl.se/libcurl/c/https.html).
/// On Linux you may need to install `pkg-config` and `libssl-dev`.
//# Purpose: Demo `curl` implementation.
use curl::easy::Easy;
use std::io::{stdout, Write};

fn main() -> Result<(), curl::Error> {
    let mut curl = Easy::new();

    curl.url("https://github.com/durbanlegend/rs-script/demo/*")?;
    curl.write_function(|data| {
        stdout().write_all(data).unwrap();
        Ok(data.len())
    })?;

    curl.perform()?;

    Ok(())
}
