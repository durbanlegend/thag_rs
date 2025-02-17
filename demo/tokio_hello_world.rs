/*[toml]
[dependencies]
tokio = { version = "1.38.0", features = ["full"] }
*/

// A simple client that opens a TCP stream, writes "hello world\n", and closes
// the connection.
//
//
//     ncat -l 6142
//
// And then in another terminal run:
//
//     cargo run --example hello_world

#![warn(rust_2018_idioms)]

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use std::error::Error;

/// Published example from `tokio` crate. Before running, start a server: `ncat -l 6142`
/// in another terminal.
//# Purpose: Demo running `tokio` from `thag_rs`.
//# Categories: async, educational, technique
#[tokio::main]
// To start a server that this client can talk to on port 6142, you can use this command:
pub async fn main() -> Result<(), Box<dyn Error>> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("created stream");

    let result = stream.write_all(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}
