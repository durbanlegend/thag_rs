/// Copied from `crokey` under MIT licence.
/// Copyright (c) 2022 Canop
///
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use strict::OneToThree;

/// A Key combination wraps from one to three standard keys with optional modifiers
/// (ctrl, alt, shift).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct KeyCombination {
    pub codes: OneToThree<KeyCode>,
    pub modifiers: KeyModifiers,
}

/// Change the char to uppercase when the modifier shift is present,
/// otherwise if the char is uppercase, return true.
/// If the key is the '\r' or '\n' char, change it to `KeyCode::Enter`.
fn normalize_key_code(code: &mut KeyCode, modifiers: KeyModifiers) -> bool {
    // profile_fn!(normalize_key_code);
    if matches!(code, KeyCode::Char('\r' | '\n')) {
        *code = KeyCode::Enter;
    } else if modifiers.contains(KeyModifiers::SHIFT) {
        if let KeyCode::Char(c) = code {
            if c.is_ascii_lowercase() {
                *code = KeyCode::Char(c.to_ascii_uppercase());
            }
        }
    } else if let KeyCode::Char(c) = code {
        if c.is_ascii_uppercase() {
            return true;
        }
    }
    false
}

impl KeyCombination {
    /// Return a normailzed version of the combination.
    ///
    /// Fix the case of the code to uppercase if the shift modifier is present.
    /// Add the SHIFT modifier if one code is uppercase.
    ///
    /// This allows direct comparisons with the fields of `crossterm::event::KeyEvent`
    /// whose code is uppercase when the shift modifier is present. And supports the
    /// case where the modifier isn't mentioned but the key is uppercase.
    #[must_use]
    pub fn normalized(mut self) -> Self {
        // profile_method!(normalized);
        let mut shift = normalize_key_code(self.codes.first_mut(), self.modifiers);
        if let Some(ref mut code) = self.codes.get_mut(1) {
            shift |= normalize_key_code(code, self.modifiers);
        }
        if let Some(ref mut code) = self.codes.get_mut(2) {
            shift |= normalize_key_code(code, self.modifiers);
        }
        if shift {
            self.modifiers |= KeyModifiers::SHIFT;
        }
        self
    }
}

impl From<KeyEvent> for KeyCombination {
    fn from(key_event: KeyEvent) -> Self {
        // profile_method!(from);
        let raw = Self {
            codes: key_event.code.into(),
            modifiers: key_event.modifiers,
        };
        raw.normalized()
    }
}

/// A macro that calls the private `key` proc macro to create a `KeyCombination` from an idiomatic shorthand.
/// Directly borrowed from the `crokey` crate.
#[macro_export]
macro_rules! key {
    ($($tt:tt)*) => {
        $crate::__private::key!(($crate) $($tt)*)
    };
}
