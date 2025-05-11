/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["full_profiling"] }
*/

//! A minimal chat server example with proper profiling.
//! 
//! Connect using: nc localhost 6000

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thag_profiler::{enable_profiling, profiled, ProfileConfiguration, ProfileType};

#[derive(Clone)]
struct Server {
    clients: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>,
}

impl Server {
    fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[profiled]
    fn broadcast(&self, message: &str) {
        let clients = self.clients.lock().unwrap();
        for client in clients.values() {
            let mut client = client.lock().unwrap();
            let _ = client.write_all(message.as_bytes());
            let _ = client.flush();
        }
    }

    #[profiled]
    fn handle_client(&self, stream: TcpStream) {
        let addr = stream.peer_addr().unwrap();
        println!("New client connected: {}", addr);
        
        // Add client to the list
        let stream_clone = Arc::new(Mutex::new(stream));
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(addr, stream_clone.clone());
        }
        
        // Announce the connection
        self.broadcast(&format!("{} has joined the chat\n", addr));
        
        // Handle client messages
        let stream_for_reading = stream_clone.lock().unwrap();
        let mut reader = BufReader::new(&*stream_for_reading);
        let mut buffer = String::new();
        
        while let Ok(bytes_read) = reader.read_line(&mut buffer) {
            if bytes_read == 0 {
                break; // Client disconnected
            }
            
            let message = format!("{}: {}", addr, buffer);
            println!("{}", message);
            
            // Broadcast to all clients
            self.broadcast(&message);
            
            buffer.clear();
        }
        
        // Remove client and announce departure
        {
            let mut clients = self.clients.lock().unwrap();
            clients.remove(&addr);
        }
        
        self.broadcast(&format!("{} has left the chat\n", addr));
        println!("Client disconnected: {}", addr);
    }
}

#[enable_profiling]
fn main() {
    // Initialize profiling
    let mut config = ProfileConfiguration::default();
    config.set_profile_type(ProfileType::Both);
    thag_profiler::init_profiling("minimal_chat", config);
    
    println!("Initializing minimal chat server...");
    
    // Create server
    let server = Server::new();
    let listener = TcpListener::bind("127.0.0.1:6000").expect("Failed to bind to address");
    println!("Server listening on 127.0.0.1:6000");
    println!("Connect with: nc localhost 6000");
    println!("Server will run for 60 seconds then exit");
    
    // Create a thread to auto-shutdown the server
    let shutdown_flag = Arc::new(Mutex::new(false));
    let shutdown_flag_clone = shutdown_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(60));
        println!("\nShutdown time reached, stopping server...");
        *shutdown_flag_clone.lock().unwrap() = true;
    });
    
    // Accept clients with timeout
    listener.set_nonblocking(true).expect("Failed to set non-blocking");
    
    // Handle client connections
    let mut handle_threads = vec![];
    
    'accept_loop: loop {
        // Check if we should shutdown
        if *shutdown_flag.lock().unwrap() {
            break 'accept_loop;
        }
        
        // Accept with timeout
        match listener.accept() {
            Ok((stream, _)) => {
                let server_clone = server.clone();
                let handle = thread::spawn(move || {
                    server_clone.handle_client(stream);
                });
                handle_threads.push(handle);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection available, sleep briefly
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                break;
            }
        }
    }
    
    println!("Server shutting down, waiting for client threads...");
    
    // Wait for client threads to finish with timeout
    for (i, handle) in handle_threads.iter().enumerate() {
        match handle.join() {
            Ok(_) => println!("Client thread {} completed", i),
            Err(_) => println!("Client thread {} panicked", i),
        }
    }
    
    // Finalize profiling
    println!("Finalizing profiling data...");
    thag_profiler::finalize_profiling();
    println!("Server shutdown complete!");
}