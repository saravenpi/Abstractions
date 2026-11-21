//! The names of the keys that print nothing, and the name of a key by its place.

use bevy::input::keyboard::Key;
use bevy::prelude::KeyCode;

use super::known::ORDER;

/// Most characters a label has: what the value column of the settings holds beside the mark of a
/// conflict.
pub const LABEL_CHARS: usize = 12;

/// The keys that print nothing, each with its name in words. Where the side matters, the name
/// says it.
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
    (KeyCode::Backspace, "Backspace"),
    (KeyCode::Delete, "Del"),
    (KeyCode::Insert, "Ins"),
    (KeyCode::Home, "Home"),
    (KeyCode::End, "End"),
    (KeyCode::PageUp, "Page Up"),
    (KeyCode::PageDown, "Page Down"),
    (KeyCode::CapsLock, "Caps Lock"),
    (KeyCode::ContextMenu, "Menu"),
    (KeyCode::NumpadEnter, "Num Enter"),
];

/// What a Mac prints on the keys it names differently.
const MAC: [(KeyCode, &str); 4] = [
    (KeyCode::AltLeft, "Left Option"),
    (KeyCode::AltRight, "Right Option"),
    (KeyCode::SuperLeft, "Left Cmd"),
    (KeyCode::SuperRight, "Right Cmd"),
];

/// Returns the name of `key` in `names`.
fn find(names: &[(KeyCode, &'static str)], key: KeyCode) -> Option<&'static str> {
    names
        .iter()
        .find(|(code, _)| *code == key)
        .map(|(_, name)| *name)
}

/// Returns the name in words of a key that prints nothing, whatever the layout.
pub(super) fn named(key: KeyCode) -> Option<&'static str> {
    let mac = cfg!(target_os = "macos").then(|| find(&MAC, key)).flatten();
    mac.or_else(|| find(&NAMES, key))
}

/// Returns the name of a key by its place, for a key no layout names: its code name without the
/// words every key shares, such as F5 or Num 1.
pub(super) fn place(key: KeyCode) -> String {
    let name = format!("{key:?}");
    let short = ["Key", "Digit"]
        .iter()
        .find_map(|prefix| name.strip_prefix(prefix));
    let short = short.unwrap_or(&name).replace("Numpad", "Num ");
    short.chars().take(LABEL_CHARS).collect()
}

/// Returns `c` in upper case when that is one character, as it is printed on a key. A letter
/// with no single capital, such as the German sharp s, stays as it is.
fn capital(c: char) -> char {
    let mut upper = c.to_uppercase();
    match (upper.next(), upper.next()) {
        (Some(one), None) => one,
        _ => c,
    }
}

/// Returns the one character `logical` prints: the character of a key that types one, or the
/// accent of a dead key.
fn typed(logical: &Key) -> Option<char> {
    match logical {
        Key::Character(text) => {
            let mut chars = text.chars();
            chars.next().filter(|_| chars.next().is_none())
        }
        Key::Dead(accent) => *accent,
        _ => None,
    }
}

/// Returns what a press teaches: the place of the printing key `key` in [`ORDER`] and what
/// `logical` prints on it. A key of the number row keeps its digit, which is printed on it in
/// every layout: it teaches nothing while it types a symbol, as on AZERTY.
pub(super) fn printed(key: KeyCode, logical: &Key) -> Option<(usize, char)> {
    let at = ORDER.iter().position(|code| *code == key)?;
    let c = typed(logical).filter(|c| !c.is_control() && !c.is_whitespace())?;
    let digit_row = (1..=10).contains(&at);
    (!digit_row || c.is_ascii_digit()).then(|| (at, capital(c)))
}
