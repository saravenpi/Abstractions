//! Gameplay options: how a fight feels, apart from the picture, the controls and the sound.

use serde::{Deserialize, Serialize};

/// How large the interface is drawn, as a share of the size its window gives it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterfaceSize {
    /// Three quarters of the usual size.
    Small,
    /// The size that follows the window height.
    #[default]
    Normal,
    /// One and a quarter times the usual size.
    Large,
}

impl InterfaceSize {
    /// Every step, from the smallest.
    pub const ALL: [InterfaceSize; 3] = [Self::Small, Self::Normal, Self::Large];

    /// Returns the factor the step multiplies the scale of the window height by.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Small => 0.75,
            Self::Normal => 1.0,
            Self::Large => 1.25,
        }
    }

    /// Returns the name of the step with its share, as the settings show it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small 75%",
            Self::Normal => "Normal 100%",
            Self::Large => "Large 125%",
        }
    }

    /// Returns the step `dir` places from this one, staying on the first and the last.
    pub fn moved(self, dir: i32) -> Self {
        let at = Self::ALL.iter().position(|step| *step == self).unwrap_or(1) as i32;
        Self::ALL[(at + dir).clamp(0, 2) as usize]
    }
}

/// Gameplay options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Gameplay {
    /// Strength of the hitstop and its shake, from 0 (off) to 1 (full): the short freeze of the robot
    /// and the animal it struck at the moment of a hit.
    pub hitstop: f32,
    /// Whether pins, the course and other markers show over the world, with an arrow at the
    /// edge of the screen when they are out of view.
    pub world_markers: bool,
    /// True to show the blood of a wound: the spurts and the stains. The bleeding itself goes on
    /// either way.
    pub blood: bool,
    /// How large the interface is, the HUD and the menus together.
    pub interface: InterfaceSize,
}

impl Default for Gameplay {
    fn default() -> Self {
        Self {
            hitstop: 1.0,
            world_markers: true,
            blood: true,
            interface: InterfaceSize::Normal,
        }
    }
}
