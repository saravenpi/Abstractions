//! The words of a text that names keys, with the keys it names.

use bevy::prelude::*;

use crate::settings::{KeyLabels, Keymap};

/// One key named by a [`KeyLine`].
#[derive(Clone, Copy)]
pub enum KeySlot {
    /// A key that no setting moves.
    Fixed(KeyCode),
    /// The key bound to an action, read from the bindings.
    Bound(fn(&Keymap) -> KeyCode),
}

impl KeySlot {
    /// Returns the key of the slot under the bindings `map`.
    fn key(self, map: &Keymap) -> KeyCode {
        match self {
            KeySlot::Fixed(key) => key,
            KeySlot::Bound(of) => of(map),
        }
    }
}

impl From<KeyCode> for KeySlot {
    fn from(key: KeyCode) -> Self {
        KeySlot::Fixed(key)
    }
}

/// Returns the slot of the key bound to an action, as in `bound(|keys| keys.map)`.
pub fn bound(of: fn(&Keymap) -> KeyCode) -> KeySlot {
    KeySlot::Bound(of)
}

/// The words of a text that names keys. Each `{}` of the template is the label of the next key,
/// as the layout of the player prints it. [`write`] keeps the text of its entity up to date.
#[derive(Component, Clone)]
pub struct KeyLine {
    /// The words, with a `{}` where each key is named.
    template: &'static str,
    /// The keys named, in the order of the template.
    keys: Vec<KeySlot>,
}

impl KeyLine {
    /// Returns the line of `template` that names `keys`.
    pub fn new(template: &'static str, keys: impl IntoIterator<Item = KeySlot>) -> Self {
        Self {
            template,
            keys: keys.into_iter().collect(),
        }
    }

    /// Returns the words under the bindings `map` and the labels `labels`.
    pub fn words(&self, map: &Keymap, labels: &KeyLabels) -> String {
        let keys: Vec<KeyCode> = self.keys.iter().map(|slot| slot.key(map)).collect();
        labels.fill(self.template, &keys)
    }

    /// Returns the words under the default bindings on QWERTY: what the text holds before
    /// [`write`] first runs.
    pub(super) fn plain(&self) -> String {
        self.words(&Keymap::default(), &KeyLabels::default())
    }
}

impl From<&'static str> for KeyLine {
    fn from(words: &'static str) -> Self {
        KeyLine::new(words, [])
    }
}

impl From<KeyCode> for KeyLine {
    fn from(key: KeyCode) -> Self {
        KeyLine::new("{}", [KeySlot::Fixed(key)])
    }
}
