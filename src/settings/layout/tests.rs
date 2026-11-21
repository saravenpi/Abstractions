//! Tests of the labels of the keys: what each layout prints, what a press teaches, what a
//! change of layout does, and that no key is ever left without a name.

use bevy::input::keyboard::Key;
use bevy::prelude::KeyCode;

use super::known::{ORDER, nearest};
use super::names::LABEL_CHARS;
use super::{KeyLabels, Layout};

/// Keys of every kind: letters, digits, signs, modifiers, the pad, the function row, and keys
/// few keyboards have.
const SAMPLE: [KeyCode; 20] = [
    KeyCode::KeyW,
    KeyCode::Digit1,
    KeyCode::Slash,
    KeyCode::IntlBackslash,
    KeyCode::ShiftLeft,
    KeyCode::AltRight,
    KeyCode::SuperLeft,
    KeyCode::Space,
    KeyCode::Escape,
    KeyCode::ArrowUp,
    KeyCode::Numpad7,
    KeyCode::NumpadEnter,
    KeyCode::NumpadMemorySubtract,
    KeyCode::F5,
    KeyCode::F24,
    KeyCode::IntlRo,
    KeyCode::AudioVolumeDown,
    KeyCode::BrowserFavorites,
    KeyCode::PrintScreen,
    KeyCode::Lang1,
];

/// Returns the logical key of a key that types `text`.
fn types(text: &str) -> Key {
    Key::Character(text.into())
}

/// Returns labels of QWERTY that learned each pair of `pairs`: a key and what it types.
fn learned(pairs: &[(KeyCode, &str)]) -> KeyLabels {
    let mut labels = KeyLabels::default();
    for (key, text) in pairs {
        labels.learn(*key, &types(text));
    }
    labels
}

/// Every layout names each of the 48 printing keys with one character, and no key of any kind
/// has an empty label or one too long for the settings, in any layout.
#[test]
fn no_label_is_empty_and_every_layout_is_whole() {
    for layout in Layout::ALL {
        assert_eq!(layout.keys().chars().count(), ORDER.len(), "{layout:?}");
        let labels = KeyLabels::of(layout);
        for key in ORDER.into_iter().chain(SAMPLE) {
            let label = labels.label(key);
            let chars = label.chars().count();
            assert!((1..=LABEL_CHARS).contains(&chars), "{layout:?} {key:?}");
            assert_eq!(label.trim(), label, "{layout:?} {key:?}");
        }
    }
    let odd = KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified);
    assert!(!KeyLabels::default().label(odd).is_empty());
}

/// The keys of walking are W A S D on QWERTY, Z Q S D on AZERTY and other letters on the other
/// layouts, while the keys that print nothing keep their names and the number row its digits.
#[test]
fn each_layout_prints_its_own_letters() {
    let walk = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD];
    let said = |layout| KeyLabels::of(layout).fill("{}{}{}{}", &walk);
    assert_eq!(said(Layout::Us), "WASD");
    assert_eq!(said(Layout::Azerty), "ZQSD");
    assert_eq!(said(Layout::Qwertz), "WASD");
    assert_eq!(said(Layout::Dvorak), ",AOE");
    assert_eq!(said(Layout::Colemak), "WARS");
    assert_eq!(said(Layout::Bepo), "ÉAUI");
    let french = KeyLabels::of(Layout::Azerty);
    assert_eq!(french.label(KeyCode::Digit1), "1");
    assert_eq!(french.label(KeyCode::KeyM), ",");
    assert_eq!(french.label(KeyCode::Semicolon), "M");
    assert_eq!(french.label(KeyCode::ShiftLeft), "Left Shift");
    assert_eq!(french.label(KeyCode::Space), "Space");
    assert_eq!(KeyLabels::of(Layout::Qwertz).label(KeyCode::KeyY), "Z");
    assert_eq!(KeyLabels::default().label(KeyCode::F5), "F5");
    assert_eq!(KeyLabels::default().label(KeyCode::Numpad7), "Num 7");
}

/// One press of a key that tells the layout names the keys not pressed yet: Z on the key of W
/// is AZERTY, and the keys of A and M follow. A key that types a symbol on the number row keeps
/// its digit.
#[test]
fn a_press_teaches_its_key_and_the_layout() {
    let mut labels = KeyLabels::default();
    assert!(labels.learn(KeyCode::KeyW, &types("z")));
    assert!(!labels.learn(KeyCode::KeyW, &types("z")), "nothing new");
    assert_eq!(labels.layout(), Layout::Azerty);
    assert_eq!(labels.label(KeyCode::KeyW), "Z");
    assert_eq!(labels.label(KeyCode::KeyA), "Q");
    assert_eq!(labels.label(KeyCode::KeyM), ",");
    assert!(!labels.learn(KeyCode::Digit1, &types("&")));
    assert_eq!(labels.label(KeyCode::Digit1), "1");
    assert!(!labels.learn(KeyCode::Space, &Key::Space));
    assert!(!labels.learn(KeyCode::KeyE, &types("  ")));
    let german = learned(&[(KeyCode::KeyY, "z"), (KeyCode::Minus, "ß")]);
    assert_eq!(german.layout(), Layout::Qwertz);
    assert_eq!(german.label(KeyCode::KeyZ), "Y");
    assert_eq!(german.label(KeyCode::Minus), "ß");
    let dvorak = learned(&[(KeyCode::KeyW, ","), (KeyCode::KeyS, "o")]);
    assert_eq!(dvorak.layout(), Layout::Dvorak);
    assert_eq!(dvorak.label(KeyCode::KeyD), "E");
}

/// A layout the game does not know is still named key by key as its keys are pressed, a dead
/// key shows its accent, and the keys not pressed keep the nearest known layout.
#[test]
fn an_unknown_layout_is_learned_key_by_key() {
    let mut labels = KeyLabels::default();
    labels.learn(KeyCode::KeyW, &types("ж"));
    labels.learn(KeyCode::BracketLeft, &Key::Dead(Some('~')));
    assert_eq!(labels.label(KeyCode::KeyW), "Ж");
    assert_eq!(labels.label(KeyCode::BracketLeft), "~");
    assert_eq!(labels.label(KeyCode::KeyA), "A");
    assert_eq!(labels.layout(), Layout::Us);
}

/// A key that prints something else than it did means the layout changed in the session: what
/// was learned is forgotten and the labels follow the new layout.
#[test]
fn a_layout_changed_in_the_session_changes_the_labels() {
    let mut labels = learned(&[(KeyCode::KeyW, "z"), (KeyCode::KeyA, "q")]);
    assert_eq!(labels.fill("{} {}", &[KeyCode::KeyW, KeyCode::KeyA]), "Z Q");
    assert!(labels.learn(KeyCode::KeyW, &types("w")));
    assert_eq!(labels.layout(), Layout::Us);
    assert_eq!(labels.fill("{} {}", &[KeyCode::KeyW, KeyCode::KeyA]), "W A");
    let kept = nearest(&[None; 48], Layout::Bepo);
    assert_eq!(kept, Layout::Bepo, "nothing learned keeps the guess");
}
