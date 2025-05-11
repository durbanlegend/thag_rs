/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
async-channel = "1.9.0"
smol = "1.3.0"
*/

//! A simplified TCP chat server with profiling.
//!
//! Run with:
//! `thag run demo/smol_chat_server_profile_simple.rs`

use async_channel::{bounded, Receiver, Sender};
use smol::{io, prelude::*, Async};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
use thag_profiler::{profiled, ProfileConfiguration, ProfileType};

// Simple message type
enum Message {
    Join(SocketAddr, Arc<Async<TcpStream>>),
    Leave(SocketAddr),
    Chat(SocketAddr, String),
}

// Main dispatch function to handle messages
#[profiled]
async fn dispatch(rx: Receiver<Message>) -> io::Result<()> {
    println!("Starting dispatch loop");

    let mut clients = HashMap::<SocketAddr, Arc<Async<TcpStream>>>::new();

    while let Ok(msg) = rx.recv().await {
        match msg {
            Message::Join(addr, stream) => {
                println!("Client connected: {}", addr);
                clients.insert(addr, stream);

                // Broadcast join message
                let join_msg = format!("* {} has joined the chat\n", addr);
                for client in clients.values() {
                    let _ = client.write_all(join_msg.as_bytes()).await;
                }
            }

            Message::Leave(addr) => {
                println!("Client disconnected: {}", addr);
                clients.remove(&addr);

                // Broadcast leave message
                let leave_msg = format!("* {} has left the chat\n", addr);
                for client in clients.values() {
                    let _ = client.write_all(leave_msg.as_bytes()).await;
                }
            }

            Message::Chat(from, content) => {
                let chat_msg = format!("{}: {}\n", from, content);
                println!("{}", chat_msg);

                // Broadcast to all clients
                for client in clients.values() {
                    let _ = client.write_all(chat_msg.as_bytes()).await;
                }
            }
        }
    }

    println!("Dispatch loop ended");
    Ok(())
}

// Handle a single client connection
#[profiled]
async fn handle_client(client: Arc<Async<TcpStream>>, tx: Sender<Message>) -> io::Result<()> {
    let addr = client.get_ref().peer_addr()?;

    // Send join notification
    tx.send(Message::Join(addr, client.clone())).await.ok();

    // Create a buffered reader for the client stream
    let mut reader = io::BufReader::new(client.clone());
    let mut line = String::new();

    // Read messages from client
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // Connection closed
            Ok(_) => {
                // Trim the newline and send the message
                if let Some(content) = line.strip_suffix("\n") {
                    if content.trim().is_empty() {
                        continue;
                    }
                    tx.send(Message::Chat(addr, content.to_string())).await.ok();
                }
            }
            Err(_) => break,
        }
    }

    // Send leave notification
    tx.send(Message::Leave(addr)).await.ok();
    Ok(())
}

fn main() -> io::Result<()> {
    // Initialize profiling with both time and memory
    let mut config = ProfileConfiguration::default();
    config.set_profile_type(ProfileType::Both);
    thag_profiler::init_profiling("chat_server", config);

    println!("Chat server starting...");
    println!("Profiling enabled - both time and memory tracking active");

    // Set a fixed timeout for the server to automatically stop
    let server_duration = Duration::from_secs(60);
    println!(
        "Server will automatically stop and save profile data after {} seconds",
        server_duration.as_secs()
    );

    // Run the server
    smol::block_on(async {
        // Create TCP listener
        let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 6000))?;
        println!("Server listening on {}", listener.get_ref().local_addr()?);

        // Create message channel
        let (tx, rx) = bounded(100);

        // Spawn the dispatch task
        let dispatch_task = smol::spawn(dispatch(rx));

        // Set up a timer to automatically stop the server
        let shutdown_timer = smol::spawn(async move {
            smol::Timer::after(server_duration).await;
            println!("\nServer runtime limit reached, shutting down...");
        });

        // Accept connections
        let accept_task = smol::spawn(async move {
            let mut client_tasks = Vec::new();

            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let client = Arc::new(stream);
                        let client_tx = tx.clone();

                        // Spawn a task for each client
                        let task = smol::spawn(handle_client(client, client_tx));
                        client_tasks.push(task);
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                        break;
                    }
                }

                // Check if we should stop accepting connections
                // This is a non-blocking way to check
                if smol::future::poll_once(shutdown_timer.clone())
                    .await
                    .is_some()
                {
                    break;
                }
            }

            // Wait for all client tasks to complete
            for task in client_tasks {
                let _ = task.await;
            }

            // Drop the sender to signal the dispatch task to end
            drop(tx);
        });

        // Wait for the accept task to complete
        accept_task.await;

        // Wait for the dispatch task to complete
        let _ = dispatch_task.await;

        // Make sure profiling data is written
        thag_profiler::finalize_profiling();
        println!("Server shutdown complete. Profiling data has been saved.");

        Ok(())
    })
}
