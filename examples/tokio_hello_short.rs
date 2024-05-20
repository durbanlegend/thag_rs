{//! [dependencies]
//! tokio = { version = "1.37.0", features = ["full"] }

// Shortened for pasting into REPL.
//
// Start a server: ncat -l 6142

#![warn(rust_2018_idioms)]

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("created stream");

    let result = stream.write_all(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}
}
