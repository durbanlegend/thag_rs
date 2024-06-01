/*[toml]
[dependencies]
dark-light = "1.1.1"
*/

/// This seems to think even the darkest themes are light.
fn main() {
    let mode = dark_light::detect();

    match mode {
        // Dark mode
        dark_light::Mode::Dark => {}
        // Light mode
        dark_light::Mode::Light => {}
        // Unspecified
        dark_light::Mode::Default => {}
    }
    eprintln!("mode={mode:#?}");
}
