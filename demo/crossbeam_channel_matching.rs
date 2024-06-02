/*[toml]
[dependencies]
crossbeam-channel = "0.5.13"
crossbeam-utils = "0.8.20"
*/

/// crossbeam-channel published example
/// Using `select!` to send and receive on the same channel at the same time.
use crossbeam_channel::{bounded, select};
use crossbeam_utils::thread;

fn main() {
    let people = vec!["Anna", "Bob", "Cody", "Dave", "Eva"];
    let (s, r) = bounded(1); // Make room for one unmatched send.

    // Either send my name into the channel or receive someone else's, whatever happens first.
    let seek = |name, s, r| {
        select! {
            recv(r) -> peer => println!("{} received a message from {}.", name, peer.unwrap()),
            send(s, name) -> _ => {}, // Wait for someone to receive my message.
        }
    };

    thread::scope(|scope| {
        for name in people {
            let (s, r) = (s.clone(), r.clone());
            scope.spawn(move |_| seek(name, s, r));
        }
    })
    .unwrap();

    // Check if there is a pending send operation.
    if let Ok(name) = r.try_recv() {
        println!("No one received {}’s message.", name);
    }
}
