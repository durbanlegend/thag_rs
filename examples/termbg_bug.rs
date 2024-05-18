/*[toml]
[dependencies]
termbg = "0.5.0"
*/

use std::io::{self, Read};

fn main() {
    let timeout = std::time::Duration::from_millis(100);

    // let term = termbg::terminal();
    let rgb = termbg::rgb(timeout);
    // let theme = termbg::theme(timeout);

    println!("Type in something and see if first character gets swallowed in Windows Terminal");
    let mut buffer = String::new();
    io::stdin().lock().read_to_string(&mut buffer).unwrap();
    println!("buffer={buffer:?}");

    // match rgb {
    //     Ok(rgb) => {
    //         println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
    //     }
    //     Err(e) => {
    //         println!("  Color: detection failed {:?}", e);
    //     }
    // }

    // match theme {
    //     Ok(theme) => {
    //         println!("  Theme: {:?}", theme);
    //     }
    //     Err(e) => {
    //         println!("  Theme: detection failed {:?}", e);
    //     }
    // }
}
