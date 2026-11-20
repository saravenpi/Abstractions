//! Gameplay options: how a fight feels, apart from the picture, the controls and the sound.

use serde::{Deserialize, Serialize};

/// Gameplay options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct Gameplay {
    /// Strength of the hitstop and its shake, from 0 (off) to 1 (full): the short freeze of the robot
    /// and the animal it struck at the moment of a hit.
    pub hitstop: f32,
}

impl Default for Gameplay {
    fn default() -> Self {
        Self { hitstop: 1.0 }
    }
}
