/*[toml]
[dependencies]
druid = "0.8.3"
reqwest = "0.12.4"
*/
/// This may seem bit of a catch-22 place to keep this program, but it's just a prototype.
extern crate druid;
extern crate reqwest;
use druid::widget::{Button, Flex, Label, TextBox};
use druid::Color;
use druid::{AppLauncher, WindowDesc};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), reqwest::Error> {
    // Function to open file dialog and get user selected path
    fn get_install_dir() -> Option<PathBuf> {
        let window = WindowDesc::new(|| {
            let mut path_text = TextBox::new();
            let mut path = "".to_string();
            let button = Button::new("Browse").on_click(move |ctx, _data, _env| {
                // Use druid's FileDialog to open directory selection dialog
                let options = druid::FileDialogOptions::new().select_directories();
                ctx.submit_window_callback(|_| {
                    let path = options.get_selected_path(&ctx.window());
                    if let Some(path) = path {
                        path_text.set_text(&path.to_string_lossy());
                        path = path.to_path_buf();
                    }
                });
            });
            let layout = Flex::column()
                .add_child(Label::new("Select installation directory:"))
                .add_child(path_text.flex_grow(1.0))
                .add_child(button);
            layout
        });
        AppLauncher::with_window(window)
            .launch(|| ())
            .expect("Failed to launch window");
        Some(path)
    }

    let install_dir = match get_install_dir() {
        Some(install_dir) => install_dir
        None => {
            println!("Download cancelled.");
            return Ok(());
        }
    };

    let url = "https://github.com/durbanlegend/rs-script/demo";
    let target_dir = install_dir.unwrap(); // Use user selected directory

    // Create the demo directory if it doesn't exist
    create_dir_all(target_dir)?;

    let client = reqwest::Client::new();
    let response = client.get(url)?.send()?;

    let mut file = File::create(format!("{}/{}.zip", target_dir, "demo_scripts"))?;
    file.write_all(&response.bytes()?)?;

    println!("Demo scripts downloaded to: {}", target_dir);

    Ok(())
}
