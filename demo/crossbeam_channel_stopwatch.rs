/*[toml]
[dependencies]
crossbeam-channel = "0.5.13"
crossbeam-utils = "0.8.20"
signal-hook = "0.3.17"
*/

/// `crossbeam-channel` published example.
///
/// Prints the elapsed time every 1 second and quits on `Ctrl+C`.
/// You can reinstate the separate main method for Windows provided you
/// run the script with the `--multimain (-m)` option.
//# Purpose: showcase featured crates.
use std::process;

fn main() {
    #[cfg(windows)]
    {
        println!("This example does not work on Windows");
        process::exit(1);
    }
    use std::io;
    use std::thread;
    use std::time::{Duration, Instant};

    use crossbeam_channel::{bounded, select, tick, Receiver};
    use signal_hook::consts::SIGINT;
    use signal_hook::iterator::Signals;

    // Creates a channel that gets a message every time `SIGINT` is signalled.
    fn sigint_notifier() -> io::Result<Receiver<()>> {
        let (s, r) = bounded(100);
        let mut signals = Signals::new([SIGINT])?;

        thread::spawn(move || {
            for _ in signals.forever() {
                if s.send(()).is_err() {
                    break;
                }
            }
        });

        Ok(r)
    }

    // Prints the elapsed time.
    fn show(dur: Duration) {
        println!("Elapsed: {}.{:03} sec", dur.as_secs(), dur.subsec_millis());
    }

    let start = Instant::now();
    let update = tick(Duration::from_secs(1));
    let ctrl_c = sigint_notifier().unwrap();

    loop {
        select! {
            recv(update) -> _ => {
                show(start.elapsed());
            }
            recv(ctrl_c) -> _ => {
                println!();
                println!("Goodbye!");
                show(start.elapsed());
                break;
            }
        }
    }
}
