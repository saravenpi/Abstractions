//! The on-disk record of one world.

use serde::{Deserialize, Serialize};

use crate::render::DayClock;

/// Everything a world file stores. Missing keys fall back to their defaults.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct WorldSave {
    /// Display name chosen by the player.
    pub name: String,
    /// Seed the whole world is generated from.
    pub seed: u64,
    /// Body position when the world was last saved, if the player ever entered it.
    pub position: Option<[f32; 3]>,
    /// Body yaw in radians.
    pub yaw: f32,
    /// Camera pitch in radians.
    pub pitch: f32,
    /// Day cycle phase from 0 to 1.
    pub day_phase: f32,
    /// Seconds spent playing this world.
    pub play_seconds: f64,
    /// Creation time in unix seconds.
    pub created: u64,
    /// Last play time in unix seconds.
    pub last_played: u64,
}

impl Default for WorldSave {
    fn default() -> Self {
        Self {
            name: "New world".to_string(),
            seed: 0,
            position: None,
            yaw: 0.0,
            pitch: 0.0,
            day_phase: DayClock::default().phase,
            play_seconds: 0.0,
            created: 0,
            last_played: 0,
        }
    }
}

/// A world file together with its id, the file stem.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldEntry {
    /// File stem of the world file.
    pub id: String,
    /// Parsed content of the file.
    pub save: WorldSave,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A save survives a trip through toml unchanged.
    #[test]
    fn round_trips_through_toml() {
        let save = WorldSave {
            name: "Isle \"one\"".to_string(),
            seed: 123_456_789_012_345_678,
            position: Some([1.5, -2.25, 30.0]),
            yaw: 0.5,
            pitch: -0.25,
            day_phase: 0.75,
            play_seconds: 5025.5,
            created: 1_700_000_000,
            last_played: 1_700_000_500,
        };
        let text = toml::to_string(&save).unwrap();
        assert_eq!(toml::from_str::<WorldSave>(&text).unwrap(), save);
    }

    /// An old file with few keys still loads, and the rest take defaults.
    #[test]
    fn partial_file_takes_defaults() {
        let save: WorldSave = toml::from_str("name = \"Old\"\nseed = 7\n").unwrap();
        assert_eq!(save.seed, 7);
        assert_eq!(save.position, None);
        assert_eq!(save.day_phase, DayClock::default().phase);
    }
}
