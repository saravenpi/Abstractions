//! The on-disk record of one world.

use bevy::math::DVec3;
use serde::{Deserialize, Serialize};

use crate::computer::Atlas;
use crate::render::DayClock;
use crate::sim::Changes;
use crate::universe::BodyId;

/// Version written into world files. Files from before v0.6 carry none and read as 0, files
/// from v0.6 to 0.8.1 carry 1, and 2 is the ground of 0.9: a moved home base and new relief.
pub const SAVE_VERSION: u32 = 2;

/// A spot on a planet: a direction from its centre and a height above its sea level.
///
/// Unlike render or chart coordinates, it means the same thing in every session.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(default)]
pub struct Place {
    /// The planet.
    pub body: BodyId,
    /// Direction from the planet centre in the frame that turns with the planet.
    pub dir: [f64; 3],
    /// Height above sea level in metres.
    pub altitude: f64,
}

impl Place {
    /// Returns the position in metres from the centre of a planet of sea level `radius`.
    ///
    /// Returns nothing when the direction has no length, as in a damaged file.
    pub fn position(&self, radius: f64) -> Option<DVec3> {
        let dir = DVec3::from(self.dir).try_normalize()?;
        Some(dir * (radius + self.altitude))
    }
}

/// Everything a world file stores. Missing keys fall back to their defaults and unknown keys
/// are ignored, so files from older and newer versions both load.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct WorldSave {
    /// Display name chosen by the player.
    pub name: String,
    /// Seed the whole world is generated from.
    pub seed: u64,
    /// Format version of the file, 0 when the file has none.
    #[serde(default)]
    pub version: u32,
    /// Body position on the legacy flat world, if the player ever entered it.
    ///
    /// A world on a globe stores [`WorldSave::place`] instead.
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
    /// Universe time in seconds, which places every planet on its orbit and spin.
    pub clock_seconds: f64,
    /// Where the player last stood on a planet.
    pub place: Option<Place>,
    /// What the player has discovered. A file without it gets the atlas of a new game.
    pub atlas: Option<Atlas>,
    /// State of the ship, written and read by the ship itself.
    pub ship: Option<toml::Value>,
    /// What the players changed, oldest first. A file from before 0.9 has an empty log.
    #[serde(default)]
    pub changes: Changes,
    /// The ground uncovered on each body, as the words of its explored bitset. A file from
    /// before 0.10.1 has none.
    #[serde(default)]
    pub explored: Vec<Explored>,
    /// The player's favourite places. A file from before 0.10.1 has none.
    #[serde(default)]
    pub marks: Vec<Favourite>,
    /// The tools and materials the robot carries. A file from before 0.13 has none.
    #[serde(default)]
    pub pack: crate::craft::pack::Pack,
    /// How whole each part of the robot is, in the order of its parts. A file from before robot
    /// health has a whole robot.
    #[serde(default)]
    pub health: Option<[f32; crate::condition::PARTS]>,
    /// The gear a destroyed robot left where it fell, if it is still there.
    #[serde(default)]
    pub cache: Option<crate::craft::fall::Cache>,
}

/// The ground uncovered on one body: a 512 by 256 grid in longitude and latitude, one bit a
/// cell, row by row from the south, packed in 64-bit words.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Explored {
    /// The body.
    pub body: BodyId,
    /// The bits, 2048 words when complete.
    pub words: Vec<u64>,
}

/// A place the player saved with a name.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Favourite {
    /// Name the player gave it.
    pub name: String,
    /// The body it is on.
    pub body: BodyId,
    /// Direction from the body centre in the frame that turns with the body.
    pub dir: [f64; 3],
}

impl Default for WorldSave {
    fn default() -> Self {
        Self {
            name: "New world".to_string(),
            seed: 0,
            version: SAVE_VERSION,
            position: None,
            yaw: 0.0,
            pitch: 0.0,
            day_phase: DayClock::default().phase,
            play_seconds: 0.0,
            created: 0,
            last_played: 0,
            clock_seconds: 0.0,
            place: None,
            atlas: None,
            ship: None,
            changes: Changes::default(),
            explored: Vec::new(),
            marks: Vec::new(),
            pack: Default::default(),
            health: None,
            cache: None,
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
            ..WorldSave::default()
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
