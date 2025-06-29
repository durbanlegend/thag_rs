/// Published example from `reedline` crate.
///
/// The latest version of this example is available in the [examples] folder in the `reedline`
/// repository. At time of writing you can run it successfully simply
/// by invoking its URL with the `thag_url` tool, like this:
///
/// ```bash
/// thag_url https://github.com/nushell/reedline/blob/main/examples/list_bindings.rs
/// ```
///
/// Obviously this requires you to have first installed `thag_rs` with the `tools` feature.
///
//# Purpose: Explore featured crate.
//# Categories: crates, REPL, technique
use reedline::{
    get_reedline_default_keybindings, get_reedline_edit_commands,
    get_reedline_keybinding_modifiers, get_reedline_keycodes, get_reedline_prompt_edit_modes,
    get_reedline_reedline_events,
};

fn main() {
    get_all_keybinding_info();
    println!();
}

/// List all keybinding information
fn get_all_keybinding_info() {
    println!("--Key Modifiers--");
    for mods in get_reedline_keybinding_modifiers().iter() {
        println!("{mods}");
    }

    println!("\n--Modes--");
    for modes in get_reedline_prompt_edit_modes().iter() {
        println!("{modes}");
    }

    println!("\n--Key Codes--");
    for kcs in get_reedline_keycodes().iter() {
        println!("{kcs}");
    }

    println!("\n--Reedline Events--");
    for rle in get_reedline_reedline_events().iter() {
        println!("{rle}");
    }

    println!("\n--Edit Commands--");
    for edit in get_reedline_edit_commands().iter() {
        println!("{edit}");
    }

    println!("\n--Default Keybindings--");
    for (mode, modifier, code, event) in get_reedline_default_keybindings() {
        println!("mode: {mode}, keymodifiers: {modifier}, keycode: {code}, event: {event}");
    }
}
