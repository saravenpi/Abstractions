//! The on-disk record of one world.

use abstractions_net::leaves::Run;
use bevy::math::DVec3;
use serde::{Deserialize, Serialize};

use crate::computer::Atlas;
use crate::life::SpeciesId;
use crate::render::DayClock;
use crate::sim::Changes;
use crate::universe::BodyId;

/// Version written into world files. Files from before v0.6 carry none and read as 0, files
/// from v0.6 to 0.8.1 carry 1, 2 is the ground of 0.9, and 3 is the universe of 0.11: real
/// planet sizes, a new generator and explored ground as leaves. A file under 3 was made by
/// 0.10 or older and opens only in 0.10.
pub const SAVE_VERSION: u32 = 3;

/// Version of the game that writes world files, as `major.minor.patch`.
pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The release that opens worlds of a version under [`SAVE_VERSION`], as `major.minor`.
pub const OLDER_VERSION: &str = "0.10";

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
    /// Version of the game that made the world, such as `0.11.0`. A file from 0.10 or older has
    /// none.
    #[serde(default)]
    pub game_version: String,
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
    /// The ground uncovered on each body, as runs of leaves. A file from before 0.10.1 has none.
    #[serde(default)]
    pub explored: Vec<Explored>,
    /// The player's pins, up to eight. Old files kept favourites here: they load as pins. A file
    /// from before 0.10.1 has none.
    #[serde(default)]
    pub marks: Vec<Pin>,
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
    /// The species the player has seen in this world. A file from before the field guide has none.
    #[serde(default)]
    pub seen: Vec<SpeciesId>,
}

/// The ground uncovered on one body: runs of the leaves of its quadtree on the cube-sphere.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Explored {
    /// The body.
    pub body: BodyId,
    /// The leaves, as runs of keys.
    pub runs: Vec<Run>,
}

/// A pin: a place the player marked on a planet, with a name. It is what old files called a
/// favourite, and the same record, so those files load as they are.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct Pin {
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
            game_version: GAME_VERSION.to_string(),
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

impl WorldSave {
    /// Returns the release that made the world, as `major.minor`, when it is an older one than
    /// this game: its generator is gone from this game, so only that release opens it.
    pub fn made_with(&self) -> Option<&'static str> {
        (self.version < SAVE_VERSION).then_some(OLDER_VERSION)
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
