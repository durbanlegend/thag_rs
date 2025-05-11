/*[toml]
[dependencies]
# thag_profiler = { git = "https://github.com/durbanlegend/thag_rs", branch = "develop", features = ["full_profiling"] }
# thag_profiler = { version = "0.1", features = ["full_profiling"] }
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/

//! A TCP chat server.
//!
//! First start a server:
//!
//! ```
//! cargo run --example chat-server
//! ```
//!
//! Then start clients:
//!
//! ```
//! cargo run --example chat-client
//! ```

use async_channel::{bounded, Receiver, Sender};
use async_dup::Arc as AsyncArc;
use signal_hook::{consts::SIGINT, flag};
use smol::{future::Future, io, prelude::*, Async, Timer};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc as StdArc,
};
use std::time::Duration;
use thag_profiler::{
    enable_profiling, /*, end, profile */
    profiled, start_periodic_profiling,
};

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
    // Set up the termination flag as an Arc for sharing between threads
    let term = StdArc::new(AtomicBool::new(false));
    // Register a handler for SIGINT
    flag::register(SIGINT, StdArc::clone(&term))?;

    // Set up periodic profiling to write data every 10 seconds
    let _profile_writer = start_periodic_profiling(Duration::from_secs(10));

    smol::block_on(async {
        // Create a listener for incoming client connections.
        let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 6000))?;

        // Intro messages.
        println!("Listening on {}", listener.get_ref().local_addr()?);
        println!("Start a chat client now!\n");
        println!("Press Ctrl+C to stop the server and save profiling data");

        // Spawn a background task that dispatches events to clients.
        let (sender, receiver) = bounded(100);
        let dispatch_task = smol::spawn(dispatch(receiver));

        // Clone term for each task that needs it
        let term_for_termination = StdArc::clone(&term);
        let term_for_accept = StdArc::clone(&term);

        // Create a task that checks the termination flag
        let termination_task = smol::spawn(async move {
            while !term_for_termination.load(Ordering::Relaxed) {
                Timer::after(Duration::from_millis(100)).await;
            }
            println!("\nReceived termination signal, shutting down...");
        });

        // Create a clone of the sender for the main thread
        let sender_for_main = sender.clone();

        let accept_task = smol::spawn(async move {
            let mut client_tasks = Vec::new();

            loop {
                if term_for_accept.load(Ordering::Relaxed) {
                    break;
                }

                // Accept the next connection, but don't block for more than 100ms
                // Use a select mechanism that's more compatible with smol
                let timeout = Timer::after(Duration::from_millis(100));
                let mut listener_accept = Box::pin(listener.accept());
                let mut timeout_future = Box::pin(timeout);

                let accept_result = smol::future::poll_fn(|cx| {
                    if let std::task::Poll::Ready(result) = listener_accept.as_mut().poll(cx) {
                        return std::task::Poll::Ready(Some(result));
                    }
                    if let std::task::Poll::Ready(_) = timeout_future.as_mut().poll(cx) {
                        return std::task::Poll::Ready(None);
                    }
                    std::task::Poll::Pending
                })
                .await;

                if let Some(Ok((stream, addr))) = accept_result {
                    // Got a connection
                    let client = AsyncArc::new(stream);
                    let sender = sender.clone();

                    // Spawn a background task reading messages from the client.
                    let client_task = smol::spawn(async move {
                        // Client starts with a `Join` event.
                        sender.send(Event::Join(addr, client.clone())).await.ok();

                        // Read messages from the client and ignore I/O errors when the client quits.
                        read_messages(sender.clone(), client).await.ok();

                        // Client ends with a `Leave` event.
                        sender.send(Event::Leave(addr)).await.ok();
                    });

                    client_tasks.push(client_task);
                } else if let Some(Err(e)) = accept_result {
                    // Error accepting connection
                    eprintln!("Error accepting connection: {}", e);
                } else {
                    // Timeout occurred, just continue the loop
                }
            }

            // Wait for client tasks to complete
            for task in client_tasks {
                task.await;
            }
        });

        // Wait for the termination signal
        termination_task.await;

        // Wait for the accept task to clean up
        accept_task.await;

        // Close the sender to terminate the dispatch task
        drop(sender_for_main);

        // Wait for dispatch to complete
        let _ = dispatch_task.await;

        // Explicitly finalize profiling data to ensure it's written
        thag_profiler::finalize_profiling();
        println!("Server shutdown complete. Profiling data has been saved.");

        Ok(())
    })
}
