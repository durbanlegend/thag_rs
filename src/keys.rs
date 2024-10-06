use crossterm::event::{KeyCode, KeyModifiers};
use strict::OneToThree;

/// A Key combination wraps from one to three standard keys with optional modifiers
/// (ctrl, alt, shift).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct KeyCombination {
    pub codes: OneToThree<KeyCode>,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone)]
pub struct KeyCombinationFormat {
    pub control: String,
    pub alt: String,
    pub shift: String,
    pub enter: String,
    pub uppercase_shift: bool,
    pub key_separator: String,
}

impl Default for KeyCombinationFormat {
    fn default() -> Self {
        Self {
            control: "Ctrl-".to_string(),
            alt: "Alt-".to_string(),
            shift: "Shift-".to_string(),
            enter: "Enter".to_string(),
            uppercase_shift: false,
            key_separator: "-".to_string(),
        }
    }
}
