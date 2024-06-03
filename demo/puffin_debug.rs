/*[toml]
[dependencies]
puffin = "0.19.0"
puffin_http = "0.16.0"
*/

use puffin;
use puffin_http::Server;
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let _puffin_server = Server::new(&server_addr).unwrap();
    eprintln!("Run this to view profiling data:  puffin_viewer {server_addr}");
    // DHF: add a delay up front to ensure it connects
    puffin::set_scopes_on(true);
    std::thread::sleep(std::time::Duration::from_secs(10));

    puffin::profile_scope!("main");

    // Call new_frame at the start of the program
    puffin::GlobalProfiler::lock().new_frame();

    let args;
    {
        puffin::profile_scope!("get_args");
        args = get_args();
    }

    {
        puffin::profile_scope!("execute");
        execute(args)?;
    }

    // Add a delay to ensure data is sent
    std::thread::sleep(std::time::Duration::from_secs(5));

    puffin::set_scopes_on(false);
    Ok(())
}

fn get_args() -> Args {
    // Your get_args implementation
    Args {}
}

fn execute(args: Args) -> Result<(), Box<dyn Error>> {
    puffin::profile_function!();
    for i in 0..1000 {
        puffin::GlobalProfiler::lock().new_frame();
        {
            puffin::profile_scope!("loop_iteration");
            // Simulate work
            eprintln!("Iteration: {}", i); // Add logging to observe progress
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
    Ok(())
}

struct Args {}
