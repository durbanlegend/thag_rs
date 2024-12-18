/*[toml]
[dependencies]
winapi = { version = "0.3.9", features = ["consoleapi", "processenv", "winbase"] }
*/

#![cfg(target_os = "windows")]
fn supports_virtual_terminal() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use winapi::um::consoleapi::GetConsoleMode;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let mut mode: u32 = 0;
        let result = unsafe { GetConsoleMode(handle, &mut mode) };
        return result != 0 && (mode & winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0);
    }
    #[cfg(not(windows))]
    {
        // On non-Windows platforms, assume support is present.
        return true;
    }
}

fn main() {
    if supports_virtual_terminal() {
        println!("Virtual terminal sequences supported!");
        // Issue sequences or use termbg here
    } else {
        println!("No virtual terminal support. Falling back.");
        // Default to your safe mode for unsupported environments
    }
}
