//! The audio settings: the three volumes, how wide the dynamic range is, and mono output.

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
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.8,
            effects: 0.8,
            range: Range::Full,
            mono: false,
        }
    }
}
