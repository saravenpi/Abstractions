//! The layouts the game knows by heart: what each prints on the 48 printing keys.

use bevy::prelude::KeyCode;

/// The printing keys in the order the rows of a layout list them: the number row, the three
/// letter rows from the top, each from the left, then the extra key of an ISO keyboard beside
/// the left Shift.
pub(super) const ORDER: [KeyCode; 48] = [
    KeyCode::Backquote,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
    KeyCode::Minus,
    KeyCode::Equal,
    KeyCode::KeyQ,
    KeyCode::KeyW,
    KeyCode::KeyE,
    KeyCode::KeyR,
    KeyCode::KeyT,
    KeyCode::KeyY,
    KeyCode::KeyU,
    KeyCode::KeyI,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::BracketLeft,
    KeyCode::BracketRight,
    KeyCode::Backslash,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::Semicolon,
    KeyCode::Quote,
    KeyCode::KeyZ,
    KeyCode::KeyX,
    KeyCode::KeyC,
    KeyCode::KeyV,
    KeyCode::KeyB,
    KeyCode::KeyN,
    KeyCode::KeyM,
    KeyCode::Comma,
    KeyCode::Period,
    KeyCode::Slash,
    KeyCode::IntlBackslash,
];

/// A keyboard layout the game knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// QWERTY of the United States.
    #[default]
    Us,
    /// QWERTY of the United Kingdom.
    Uk,
    /// AZERTY of France and Belgium.
    Azerty,
    /// QWERTZ of Germany and Austria.
    Qwertz,
    /// QWERTZ of Switzerland.
    Swiss,
    /// Dvorak.
    Dvorak,
    /// Colemak.
    Colemak,
    /// BEPO.
    Bepo,
    /// QWERTY of Sweden and Finland.
    Swedish,
    /// QWERTY of Norway.
    Norwegian,
    /// QWERTY of Denmark.
    Danish,
    /// QWERTY of Spain.
    Spanish,
    /// QWERTY of Italy.
    Italian,
}

impl Layout {
    /// Every layout. The first of two that fit the keys pressed equally well wins.
    pub const ALL: [Layout; 13] = [
        Layout::Us,
        Layout::Uk,
        Layout::Azerty,
        Layout::Qwertz,
        Layout::Swiss,
        Layout::Dvorak,
        Layout::Colemak,
        Layout::Bepo,
        Layout::Swedish,
        Layout::Norwegian,
        Layout::Danish,
        Layout::Spanish,
        Layout::Italian,
    ];

    /// Returns what the layout prints on each key of [`ORDER`], one character a key: the key
    /// pressed alone, a letter in upper case, the accent of a dead key. The number row shows its
    /// digits, which every layout prints on it even where they need Shift.
    pub(super) fn keys(self) -> &'static str {
        match self {
            Layout::Us => "`1234567890-=QWERTYUIOP[]\\ASDFGHJKL;'ZXCVBNM,./\\",
            Layout::Uk => "`1234567890-=QWERTYUIOP[]#ASDFGHJKL;'ZXCVBNM,./\\",
            Layout::Azerty => "²1234567890)=AZERTYUIOP^$*QSDFGHJKLMÙWXCVBN,;:!<",
            Layout::Qwertz => "^1234567890ß´QWERTZUIOPÜ+#ASDFGHJKLÖÄYXCVBNM,.-<",
            Layout::Swiss => "§1234567890'^QWERTZUIOPÜ¨$ASDFGHJKLÖÄYXCVBNM,.-<",
            Layout::Dvorak => "`1234567890[]',.PYFGCRL/=\\AOEUIDHTNS-;QJKXBMWVZ\\",
            Layout::Colemak => "`1234567890-=QWFPGJLUY;[]\\ARSTDHNEIO'ZXCVBKM,./\\",
            Layout::Bepo => "$1234567890=%BÉPOÈ^VDLJZWÇAUIE,CTSRNMÀYX.K'QGHFÊ",
            Layout::Swedish => "§1234567890+´QWERTYUIOPÅ¨'ASDFGHJKLÖÄZXCVBNM,.-<",
            Layout::Norwegian => "|1234567890+\\QWERTYUIOPÅ¨'ASDFGHJKLØÆZXCVBNM,.-<",
            Layout::Danish => "½1234567890+´QWERTYUIOPÅ¨'ASDFGHJKLÆØZXCVBNM,.-<",
            Layout::Spanish => "º1234567890'¡QWERTYUIOP`+ÇASDFGHJKLÑ´ZXCVBNM,.-<",
            Layout::Italian => "\\1234567890'ÌQWERTYUIOPÈ+ÙASDFGHJKLÒÀZXCVBNM,.-<",
        }
    }

    /// Returns what the layout prints on `key`, or nothing for a key that is not one of the
    /// printing keys of [`ORDER`].
    pub fn label(self, key: KeyCode) -> Option<char> {
        let at = ORDER.iter().position(|code| *code == key)?;
        self.keys().chars().nth(at)
    }
}

/// Returns the layout that fits what was `learned` best: the one that prints the most of the
/// labels seen on the keys of [`ORDER`]. `current` stays while no other fits more of them, and of
/// two that fit equally the first of [`Layout::ALL`] wins.
pub(super) fn nearest(learned: &[Option<char>; 48], current: Layout) -> Layout {
    let fits = |layout: Layout| {
        let pairs = layout.keys().chars().zip(learned);
        pairs
            .filter(|(printed, seen)| **seen == Some(*printed))
            .count()
    };
    Layout::ALL
        .into_iter()
        .fold(current, |best, layout| match fits(layout) > fits(best) {
            true => layout,
            false => best,
        })
}
