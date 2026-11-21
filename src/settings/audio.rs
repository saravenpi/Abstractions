//! The audio settings: the three volumes, how wide the dynamic range is, mono output and how often
//! the companion speaks.

use serde::{Deserialize, Serialize};

/// How far apart the quietest and the loudest moments of the game are.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Range {
    /// The mix as it is made: for headphones and good speakers in a quiet room.
    #[default]
    Full,
    /// Quiet moments raised and loud ones lowered: for laptop speakers, a noisy room or the night.
    Reduced,
}

impl Range {
    /// Returns the name of the range, as the settings show it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Reduced => "Reduced",
        }
    }

    /// Returns the other range.
    pub const fn toggled(self) -> Self {
        match self {
            Self::Full => Self::Reduced,
            Self::Reduced => Self::Full,
        }
    }
}

/// How often the companion speaks: on what matters only, now and then, or never.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Chatter {
    /// It also hums and chirps on its own, about once a minute.
    Often,
    /// It speaks on what matters, and on its own once in a few minutes.
    #[default]
    Rare,
    /// It never makes a sound.
    Off,
}

impl Chatter {
    /// Every step, in the order the settings show them.
    pub const ALL: [Chatter; 3] = [Self::Often, Self::Rare, Self::Off];

    /// Returns the name of the step, as the settings show it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Often => "Often",
            Self::Rare => "Rare",
            Self::Off => "Off",
        }
    }

    /// Returns the step `dir` places from this one, staying on the first and the last.
    pub fn moved(self, dir: i32) -> Self {
        let at = Self::ALL.iter().position(|step| *step == self).unwrap_or(1) as i32;
        Self::ALL[(at + dir).clamp(0, 2) as usize]
    }
}

/// Audio options. Each volume runs from 0.0 to 1.0.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Audio {
    /// Global volume.
    pub master: f32,
    /// Music volume.
    pub music: f32,
    /// Sound effects volume.
    pub effects: f32,
    /// How wide the dynamic range is.
    pub range: Range,
    /// Whether both ears get the same sound, for a listener who hears with one.
    pub mono: bool,
    /// How often the companion speaks.
    pub companion: Chatter,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            effects: 0.8,
            range: Range::Full,
            mono: false,
            companion: Chatter::Rare,
        }
    }
}
