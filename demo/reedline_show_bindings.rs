/*[toml]
[dependencies]
reedline = "0.35.0"
*/

/// Prototype of key binding display function for `reedline` REPL. This was developed
/// by giving ChatGPT a simple spec which it flubbed, then repeatedly feeding back errors,
/// manually corrected code and requests for changes until a nice simple display was
/// achieved. This was then refined into the `keys` display of the `thag_rs` REPL, with
/// the addition of command descriptions, non-edit commands such as SearchHistory, and colour-
/// coding.
//# Purpose: Demo the end result of development dialog with ChatGPT.
use reedline::{default_emacs_keybindings, EditCommand, KeyCode, KeyModifiers, ReedlineEvent};

fn main() {
    // Fetch default Emacs keybindings
    let key_bindings = default_emacs_keybindings();

    // Collect and format key bindings
    let mut formatted_bindings = Vec::new();
    for (key_combination, reedline_event) in key_bindings.bindings {
        let key_modifiers = key_combination.modifier;
        let key_code = key_combination.key_code;
        let modifier = format_key_modifier(key_modifiers);
        let key = format_key_code(key_code);
        let key_desc = format!("{}{}", modifier, key);
        if let ReedlineEvent::Edit(edit_cmds) = reedline_event {
            let cmd_desc = format_edit_commands(&edit_cmds);
            formatted_bindings.push((key_desc, cmd_desc));
        }
    }

    // Sort the formatted bindings alphabetically by key combination description
    formatted_bindings.sort_by(|a, b| a.0.cmp(&b.0));

    // Determine the length of the longest key description for padding
    let max_key_len = formatted_bindings
        .iter()
        .map(|(key, _)| key.len())
        .max()
        .unwrap_or(0);

    // Print the formatted and sorted key bindings
    for (key_desc, cmd_desc) in formatted_bindings {
        println!("{:<width$}    {}", key_desc, cmd_desc, width = max_key_len);
    }
}

// Helper function to convert KeyModifiers to string
fn format_key_modifier(modifier: KeyModifiers) -> String {
    let mut modifiers = Vec::new();
    if modifier.contains(KeyModifiers::CONTROL) {
        modifiers.push("CONTROL");
    }
    if modifier.contains(KeyModifiers::SHIFT) {
        modifiers.push("SHIFT");
    }
    if modifier.contains(KeyModifiers::ALT) {
        modifiers.push("ALT");
    }
    let mods_str = modifiers.join("+");
    if modifiers.len() > 0 {
        mods_str + "-"
    } else {
        mods_str
    }
}

// Helper function to convert KeyCode to string
fn format_key_code(key_code: KeyCode) -> String {
    match key_code {
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(num) => format!("F{}", num),
        KeyCode::Char(c) => format!("{}", c.to_uppercase()),
        KeyCode::Null => "Null".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::CapsLock => "CapsLock".to_string(),
        KeyCode::ScrollLock => "ScrollLock".to_string(),
        KeyCode::NumLock => "NumLock".to_string(),
        KeyCode::PrintScreen => "PrintScreen".to_string(),
        KeyCode::Pause => "Pause".to_string(),
        KeyCode::Menu => "Menu".to_string(),
        KeyCode::KeypadBegin => "KeypadBegin".to_string(),
        KeyCode::Media(media) => format!("Media({:?})", media),
        KeyCode::Modifier(modifier) => format!("Modifier({:?})", modifier),
    }
}

// Helper function to format EditCommand and include its doc comments
fn format_edit_commands(edit_cmds: &Vec<EditCommand>) -> String {
    let mut cmd_descriptions = Vec::new();
    for cmd in edit_cmds {
        let cmd_desc = match cmd {
            EditCommand::InsertNewline => {
                format!("InsertNewline: Inserts the system specific new line character/s")
            }
            // Add other EditCommand variants and their descriptions here
            _ => format!("{:?}", cmd),
        };
        cmd_descriptions.push(cmd_desc);
    }
    cmd_descriptions.join(", ")
}
