/*[toml]
[dependencies]
puffin = "0.19.0"
puffin_http = "0.16.0"
*/

use puffin::{FrameData, GlobalProfiler};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Mutex;

use profiling_data::SerializableFrameData;

#[derive(Serialize, Deserialize)]
struct SerializableFrameData {
    pub start_time: f64,
    pub duration: f64,
    pub name: String,
    pub id: u64,
}

impl From<&puffin::FrameData> for SerializableFrameData {
    fn from(frame_data: &puffin::FrameData) -> Self {
        SerializableFrameData {
            start_time: frame_data.start_time.as_secs_f64(),
            duration: frame_data.duration.as_secs_f64(),
            name: frame_data.name.clone(),
            id: frame_data.id,
        }
    }
}

mod profiling_data {
    lazy_static::lazy_static! {
        static ref PROFILER_DATA: Mutex<Vec<SerializableFrameData>> = Mutex::new(Vec::new());
    }

    pub(crate) fn save_profiling_data() {
        let profiler_data = GlobalProfiler::lock().dump_current_frame();
        let mut data = PROFILER_DATA.lock().unwrap();
        data.push((&profiler_data).into());
    }

    pub(crate) fn write_profiling_data_to_disk(filename: &str) {
        let data = PROFILER_DATA.lock().unwrap();
        let serialized_data = bincode::serialize(&*data).unwrap();
        let mut file = File::create(filename).unwrap();
        file.write_all(&serialized_data).unwrap();
    }
}

fn read_profiling_data_from_disk(filename: &str) -> Vec<SerializableFrameData> {
    let mut file = File::open(filename).unwrap();
    let mut serialized_data = Vec::new();
    file.read_to_end(&mut serialized_data).unwrap();
    bincode::deserialize(&serialized_data).unwrap()
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable profiling
    puffin::set_scopes_on(true);

    // Main profiling scope
    puffin::profile_scope!("main");

    // Placeholder for parsed arguments
    let args;
    {
        puffin::profile_scope!("get_args");
        args = get_args();
    }

    {
        puffin::profile_scope!("execute");
        execute(args)?;
    }

    // Save profiling data at intervals
    save_profiling_data();

    // Add a delay to ensure data is sent
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Write profiling data to disk
    write_profiling_data_to_disk("profiling_data.bin");

    // Disable profiling
    puffin::set_scopes_on(false);

    Ok(())
}

fn get_args() -> Args {
    puffin::profile_function!();
    std::thread::sleep(std::time::Duration::from_millis(100)); // Simulate work
    Args {}
}

fn execute(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    puffin::profile_function!();
    for i in 0..1000 {
        puffin::GlobalProfiler::lock().new_frame();
        {
            puffin::profile_scope!("loop_iteration");
            eprintln!("Iteration: {}", i); // Add logging to observe progress
            std::thread::sleep(std::time::Duration::from_millis(10)); // Simulate work
        }
    }
    save_profiling_data(); // Save profiling data after execution
    Ok(())
}

struct Args {}
