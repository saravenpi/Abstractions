//! The label of each physical key under the keyboard layout of the player.
//!
//! The game binds every action to a physical key, so the hand rests on the same keys whatever
//! the layout. What the player reads is the character their layout prints on that key: the key
//! of walking forward shows W on QWERTY and Z on AZERTY. A press teaches the pair, the rest is
//! guessed from the layout those pairs fit, and before the first press from the system.

mod guess;
mod known;
mod names;
mod param;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_events;
#[cfg(test)]
mod tests_guess;
mod watch;

pub use known::Layout;
pub use names::LABEL_CHARS;
pub use param::Labels;
pub(crate) use watch::plugin;

use bevy::input::keyboard::Key;
use bevy::prelude::*;

/// What every physical key is called on the keyboard of the player. Nothing of it is saved:
/// it is learned again in each session.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct KeyLabels {
    /// What each key of [`known::ORDER`] printed when it was last pressed alone.
    learned: [Option<char>; 48],
    /// The layout that names the keys not pressed yet.
    guess: Layout,
}

impl Default for KeyLabels {
    fn default() -> Self {
        Self::of(Layout::Us)
    }
}

impl KeyLabels {
    /// Returns the labels of `layout`, with nothing learned yet.
    pub const fn of(layout: Layout) -> Self {
        Self {
            learned: [None; 48],
            guess: layout,
        }
    }

    /// Returns the labels of the layout the system is set to, as far as it can be read without
    /// a key press. See [`guess::detect`].
    pub fn detect() -> Self {
        Self::of(guess::detect())
    }

    /// Returns the layout that names the keys not pressed yet.
    pub fn layout(&self) -> Layout {
        self.guess
    }

    /// Returns what the player reads on the physical key `key`: a name in words for a key that
    /// prints nothing, else what the key printed when pressed, else what the guessed layout
    /// prints there, else the name of its place. It is never empty and never longer than
    /// [`LABEL_CHARS`].
    pub fn label(&self, key: KeyCode) -> String {
        if let Some(name) = names::named(key) {
            return name.to_string();
        }
        let at = known::ORDER.iter().position(|code| *code == key);
        let seen = at.and_then(|at| self.learned[at]);
        match seen.or_else(|| self.guess.label(key)) {
            Some(printed) => printed.to_string(),
            None => names::place(key),
        }
    }

    /// Returns `template` with each `{}` replaced by the label of the next key of `keys`.
    pub fn fill(&self, template: &str, keys: &[KeyCode]) -> String {
        let mut parts = template.split("{}");
        let mut out = parts.next().unwrap_or_default().to_string();
        for (part, key) in parts.zip(keys) {
            out.push_str(&self.label(*key));
            out.push_str(part);
        }
        out
    }

    /// Takes in that the physical key `key`, pressed alone, means `logical`, and returns true
    /// when a label changed. A key that now prints something else means the layout changed:
    /// everything learned before is forgotten.
    pub fn learn(&mut self, key: KeyCode, logical: &Key) -> bool {
        let Some((at, printed)) = names::printed(key, logical) else {
            return false;
        };
        match self.learned[at] {
            Some(seen) if seen == printed => return false,
            Some(_) => self.learned = [None; 48],
            None => {}
        }
        self.learned[at] = Some(printed);
        self.guess = known::nearest(&self.learned, self.guess);
        true
    }
}
