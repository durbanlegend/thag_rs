{
/*[toml]
[dependencies]
tokio = { version = "1.38.0", features = ["full"] }
*/

#![warn(rust_2018_idioms)]

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use std::error::Error;

/// Published example from `tokio` crate, with comments removed to work with `thag_rs` `repl` feature.
/// Before running, start a server: `ncat -l 6142` in another terminal.
//# Purpose: Demo running `tokio` from `thag_rs`.
//# Categories: async, educational, technique
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("created stream");

    let result = stream.write_all(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}
}
