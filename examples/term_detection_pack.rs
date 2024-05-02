/*[toml]
[dependencies]
dark-light = "1.1.1"
supports-color= "3.0.0"
termbg = "0.5.0"
terminal-light = "1.4.0"
*/

use supports_color::Stream;

let timeout = std::time::Duration::from_millis(100);

println!("Termbg:");

println!("Check terminal background color");
let term = termbg::terminal();
let rgb = termbg::rgb(timeout);
let theme = termbg::theme(timeout);

println!("  Term : {:?}", term);

match rgb {
    Ok(rgb) => {
        println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
    }
    Err(e) => {
        println!("  Color: detection failed {:?}", e);
    }
}

match theme {
    Ok(theme) => {
        println!("  Theme: {:?}", theme);
    }
    Err(e) => {
        println!("  Theme: detection failed {:?}", e);
    }
}

let mode = dark_light::detect();

match mode {
    // Dark mode
    dark_light::Mode::Dark => {}
    // Light mode
    dark_light::Mode::Light => {}
    // Unspecified
    dark_light::Mode::Default => {}
}
println!("\nRust-dark-light: mode={mode:#?}");

println!("\nTerminal_light:");

let luma = terminal_light::luma();
println!("luma={luma:#?}");
match luma {
    Ok(luma) if luma > 0.5 => {
        // Use a "light mode" skin.
        println!("Light mode");
    }
    Ok(luma) if luma < 0.5 => {
        // Use a "dark mode" skin.
        println!("Dark mode");
    }
    _ => {
        // Either we couldn't determine the mode or it's kind of medium.
        // We should use an intermediate skin, or one defining the background.
        println!("Intermediate mode");
    }
}

match terminal_light::background_color()
    .map(|c| c.rgb()) {
        Ok(bg_rgb) => 
 {
let luma_255 = 0.2126 * (bg_rgb.r as f32) + 0.7152 * (bg_rgb.g as f32) + 0.0722 * (bg_rgb.b as f32);
let luma_0_to_1 = luma_255 / 255.0;
println!("\nTerminal-light: Background color is {bg_rgb:#?}, luma_255={luma_255}, luma_0_to_1={luma_0_to_1}");
}
Err(_) => println!("terminal_light::background_color() not supported"),    }

println!("\nSupports-color:");

if let Some(support) = supports_color::on(Stream::Stdout) {
    if support.has_16m {
        println!("16 million (RGB) colors are supported");
    } else if support.has_256 {
        println!("256 colors are supported.");
    } else if support.has_basic {
        println!("Only basic ANSI colors are supported.");
    }
} else {
    println!("No color support.");
}
