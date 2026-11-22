//! Readable names of keys: the full name, and the name a value column shows, cut short with an
//! ellipsis when the key has a long name nobody gave a shorter form.

use bevy::prelude::KeyCode;

use crate::ui::theme;

/// Most characters of a key name: the value column less the two of the conflict mark.
pub const KEY_CHARS: usize = theme::VALUE_CHARS - 2;

/// The mark that ends a key name cut short.
pub const ELLIPSIS: char = '\u{2026}';

/// Readable names of keys whose code name is long or unclear.
const NAMES: [(KeyCode, &str); 26] = [
    (KeyCode::ShiftLeft, "Left Shift"),
    (KeyCode::ShiftRight, "Right Shift"),
    (KeyCode::ControlLeft, "Left Ctrl"),
    (KeyCode::ControlRight, "Right Ctrl"),
    (KeyCode::AltLeft, "Left Alt"),
    (KeyCode::AltRight, "Right Alt"),
    (KeyCode::SuperLeft, "Left Super"),
    (KeyCode::SuperRight, "Right Super"),
    (KeyCode::ArrowUp, "Up"),
    (KeyCode::ArrowDown, "Down"),
    (KeyCode::ArrowLeft, "Left"),
    (KeyCode::ArrowRight, "Right"),
    (KeyCode::Space, "Space"),
    (KeyCode::Enter, "Enter"),
    (KeyCode::Escape, "Esc"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::Delete, "Del"),
    (KeyCode::Insert, "Ins"),
    (KeyCode::PageUp, "Page Up"),
    (KeyCode::PageDown, "Page Down"),
    (KeyCode::CapsLock, "Caps Lock"),
    (KeyCode::NumLock, "Num Lock"),
    (KeyCode::ScrollLock, "Scroll Lock"),
    (KeyCode::PrintScreen, "Print Screen"),
    (KeyCode::Backquote, "`"),
    (KeyCode::NumpadEnter, "Num Enter"),
];

/// Returns `full` when it fits [`KEY_CHARS`], or its start and an ellipsis.
pub fn short(full: &str) -> String {
    if full.chars().count() <= KEY_CHARS {
        return full.to_string();
    }
    let kept: String = full.chars().take(KEY_CHARS - 1).collect();
    format!("{}{ELLIPSIS}", kept.trim_end())
}

/// Returns the full readable name of a key, however long.
pub fn key_label(key: KeyCode) -> String {
    if let Some((_, name)) = NAMES.iter().find(|(code, _)| *code == key) {
        return name.to_string();
    }
    let name = format!("{key:?}");
    let short = ["Key", "Digit"]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix));
    short.unwrap_or(&name).replace("Numpad", "Num ")
}

/// Returns the name of a key as a value shows it: its readable name, or when that is longer than
/// [`KEY_CHARS`], its start and an ellipsis. The row of a key cut short names it in full in its
/// tooltip.
pub fn key_name(key: KeyCode) -> String {
    short(&key_label(key))
}
