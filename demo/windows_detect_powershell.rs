use std::env;
/// Prototype of PowerShell detection
//# Purpose: Detect if we're running under PowerShell.
//# Categories: detection, terminal, testing, windows
use std::io::{self, Read};
use sysinfo::{ProcessesToUpdate, System};

fn is_powershell() -> bool {
    // Heuristic: check PSModulePath first
    if let Ok(shell) = env::var("PSModulePath") {
        if !shell.is_empty() {
            return true;
        }
    }

    // Fallback: inspect parent process
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    if let Ok(pid) = sysinfo::get_current_pid() {
        if let Some(current) = sys.process(pid) {
            if let Some(parent_pid) = current.parent() {
                if let Some(parent) = sys.process(parent_pid) {
                    let name = parent.name();
                    if let Some(name_str) = name.to_str() {
                        let name_lower = name_str.to_ascii_lowercase();
                        return name_lower.contains("powershell") || name_lower.contains("pwsh");
                    }
                }
            }
        }
    }

    false
}

fn main() {
    if is_powershell() {
        println!("Running under PowerShell");
    } else {
        println!("Not running under PowerShell");
    }
}
