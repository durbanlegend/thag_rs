/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
smol = "1.3.0"
async-channel = "1.9.0"
async-dup = "1.2.2"
futures-lite = "1.13.0"
*/

//! A TCP chat server with profiling instrumentation and clean shutdown.
//!
//! First start the server, then connect with:
//!
//! ```
//! nc localhost 6000
//! ```
//!
//! The server will automatically shut down after 30 seconds.

use async_channel::{bounded, Receiver, Sender};
use async_dup::Arc as AsyncArc;
use smol::{io, prelude::*, Async, Timer};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled};

/// An event on the chat server.
enum Event {
    /// A client has joined.
    Join(SocketAddr, AsyncArc<Async<TcpStream>>),

    /// A client has left.
    Leave(SocketAddr),

    /// A client sent a message.
    Message(SocketAddr, String),
}

/// Dispatches events to clients.
#[profiled]
async fn dispatch(receiver: Receiver<Event>) -> io::Result<()> {
    println!("Dispatch task started");
    // Currently active clients.
    let mut map = HashMap::<SocketAddr, AsyncArc<Async<TcpStream>>>::new();

    // Receive incoming events.
    while let Ok(event) = receiver.recv().await {
        // Process the event and format a message to send to clients.
        let output = match event {
            Event::Join(addr, stream) => {
                map.insert(addr, stream);
                format!("{} has joined\n", addr)
            }
            Event::Leave(addr) => {
                map.remove(&addr);
                format!("{} has left\n", addr)
            }
            Event::Message(addr, msg) => format!("{} says: {}\n", addr, msg),
        };

        // Display the event in the server process.
        print!("{}", output);

        // Send the event to all active clients.
        for stream in map.values_mut() {
            // Ignore errors because the client might disconnect at any point.
            stream.write_all(output.as_bytes()).await.ok();
        }
    }
    println!("Dispatch task ended");
    Ok(())
}

/// Reads messages from the client and forwards them to the dispatcher task.
#[profiled]
async fn read_messages(
    sender: Sender<Event>,
    client: AsyncArc<Async<TcpStream>>,
) -> io::Result<()> {
    let addr = client.get_ref().peer_addr()?;
    let mut lines = io::BufReader::new(client).lines();

    while let Some(line) = lines.next().await {
        let line = line?;
        sender.send(Event::Message(addr, line)).await.ok();
    }
    Ok(())
}

#[enable_profiling]
fn main() -> io::Result<()> {
    // Create a simple atomic flag for shutdown coordination
    let term = Arc::new(AtomicBool::new(false));

    // Set a timer for auto-shutdown instead of relying on signals
    let shutdown_time = Duration::from_secs(30);
    println!("Chat server starting with profiling enabled...");
    println!(
        "Server will automatically stop after {} seconds",
        shutdown_time.as_secs()
    );

    smol::block_on(async {
        // Create a listener for incoming client connections.
        let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 6000))?;

        // Intro messages.
        println!("Listening on {}", listener.get_ref().local_addr()?);
        println!("Start a chat client with: nc localhost 6000\n");

        // Spawn a background task that dispatches events to clients.
        let (sender, receiver) = bounded(100);
        let dispatch_task = smol::spawn(dispatch(receiver));

        // Clone term for termination task
        let term_clone = Arc::clone(&term);

        // Create a task that sets the termination flag after a timeout
        let termination_task = smol::spawn(async move {
            // Wait for the shutdown time
            Timer::after(shutdown_time).await;
            // Set the termination flag
            term_clone.store(true, Ordering::SeqCst);
            println!("\nShutdown timer reached, server stopping...");
        });

        // Create a clone of the sender for the main thread
        let sender_for_main = sender.clone();

        // Accept incoming connections
        let accept_task = smol::spawn(async move {
            let mut client_tasks = Vec::new();

            while !term.load(Ordering::SeqCst) {
                // Try accepting a connection but time out if it takes too long
                let accept_future = listener.accept();
                let result = futures_lite::future::poll_once(async { accept_future.await }).await;

                // Check for timeout
                let timeout = Timer::after(Duration::from_millis(100));
                let timeout_elapsed = futures_lite::future::poll_once(timeout).await.is_some();

                if !timeout_elapsed {
                    if let Some(Ok((stream, addr))) = result {
                        println!("New client connected: {}", addr);

                        // Create a shareable client
                        let client: AsyncArc<Async<TcpStream>> = AsyncArc::new(stream);
                        let sender = sender.clone();

                        // Spawn a background task reading messages from the client.
                        let client_task = smol::spawn(async move {
                            // Client starts with a `Join` event.
                            sender.send(Event::Join(addr, client.clone())).await.ok();

                            // Read messages from the client and ignore I/O errors when the client quits.
                            read_messages(sender.clone(), client).await.ok();

                            // Client ends with a `Leave` event.
                            sender.send(Event::Leave(addr)).await.ok();

                            println!("Client disconnected: {}", addr);
                        });

                        client_tasks.push(client_task);
                    } else if let Some(Err(e)) = result {
                        eprintln!("Error accepting connection: {}", e);
                    }
                }

                // Small sleep to prevent busy waiting
                smol::Timer::after(Duration::from_millis(10)).await;
            }

            println!("Accept loop terminated, waiting for client tasks...");

            // Wait for client tasks to complete with a timeout
            for (i, mut task) in client_tasks.into_iter().enumerate() {
                // Use poll_once to try waiting for the task once
                let completed = futures_lite::future::poll_once(&mut task).await.is_some();

                if completed {
                    println!("Client task {i} completed");
                } else {
                    println!(
                        "Client task {i} is still running, leaving it to complete in background"
                    );
                }
            }

            println!("All client tasks processed");
        });

        // Wait for the termination timer
        termination_task.await;
        println!("Termination task completed");

        // Wait for the accept task to clean up
        accept_task.await;
        println!("Accept task completed");

        // Close the sender to terminate the dispatch task
        drop(sender_for_main);

        // Wait for dispatch with timeout
        let mut dispatch_task_mut = dispatch_task;
        let completed = futures_lite::future::poll_once(&mut dispatch_task_mut).await;

        if completed.is_some() {
            println!("Dispatch task completed successfully");
        } else {
            println!("Dispatch task still running, continuing shutdown");
        }

        // Profiling data will be finalized automatically
        // thanks to the #[enable_profiling] attribute
        println!("Server shutdown complete. Profiling data has been saved.");

        Ok(())
    })
}
