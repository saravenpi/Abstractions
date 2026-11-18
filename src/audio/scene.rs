//! The audio scene: the plain numbers every continuous sound follows.

use bevy::prelude::*;

use crate::universe::PlanetClass;

/// Seed of the piece played on the title screen, before any world exists.
pub const TITLE_SEED: u64 = 0x00AB_5712_AC71_0115;

/// What the listener's surroundings sound like right now.
///
/// The audio module fills the fields it can read from the world itself every frame while the game
/// is played, `throttle`, `dynamic_pressure` and `warping` from the instruments of the ship, and
/// `rush`, `boost` and `flutter` from the flight gear, `rain` and `dust` from the weather.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct AudioScene {
    /// Seed of the planet whose music plays.
    pub planet_seed: u64,
    /// Whether that planet is the home planet, which gets the warm variant of its piece.
    pub home: bool,
    /// The class of that planet: it sets the mood of the piece and what its shore sounds like.
    pub class: PlanetClass,
    /// How much air is left around the listener: 1 on the ground, 0 at the top of the atmosphere.
    pub altitude_fade: f32,
    /// Whether the listener is in space, outside any atmosphere.
    pub in_space: bool,
    /// Whether the ship is in warp.
    pub warping: bool,
    /// Daylight from 0 (night) to 1 (full day).
    pub daylight: f32,
    /// How far the listener's head is under water, from 0 to 1.
    pub submerged: f32,
    /// Wind strength from 0 to 1.
    pub wind: f32,
    /// How close the shoreline is, from 0 (far) to 1 (at the water's edge).
    pub shore: f32,
    /// Where the shoreline is for the listener, from -1 (left) to 1 (right).
    pub shore_side: f32,
    /// Where the companion is for the listener: its side from -1 (left) to 1 (right) in x, and
    /// how near it is from 0 (out of earshot) to 1 (beside the head) in y.
    pub companion: Vec2,
    /// Engine throttle from 0 to 1.
    pub throttle: f32,
    /// Dynamic pressure on the hull from 0 to 1, 1 being the strongest buffeting.
    pub dynamic_pressure: f32,
    /// How deep the listener is inside the cloud layer, from 0 (clear air) to 1 (its middle).
    pub cloud_depth: f32,
    /// 1 while the listener sits in the ship cabin, 0 outside.
    pub cabin: f32,
    /// Rush of the air past the flying body, from 0 (still) to 1 (very fast).
    pub rush: f32,
    /// Push of the boot boosters from 0 to 1.
    pub boost: f32,
    /// Flutter of the wingsuit cloth from 0 to 1.
    pub flutter: f32,
    /// How hard it rains on the ground and on the hull, from 0 to 1.
    pub rain: f32,
    /// How much dust the wind blows past, from 0 to 1.
    pub dust: f32,
}

impl Default for AudioScene {
    /// The title screen: the title piece at full level, by day, nothing else.
    fn default() -> Self {
        Self {
            planet_seed: TITLE_SEED,
            home: true,
            class: PlanetClass::Island,
            altitude_fade: 1.0,
            in_space: false,
            warping: false,
            daylight: 1.0,
            submerged: 0.0,
            wind: 0.0,
            shore: 0.0,
            shore_side: 0.0,
            companion: Vec2::Y,
            throttle: 0.0,
            dynamic_pressure: 0.0,
            cloud_depth: 0.0,
            cabin: 0.0,
            rush: 0.0,
            boost: 0.0,
            flutter: 0.0,
            rain: 0.0,
            dust: 0.0,
        }
    }
}

impl AudioScene {
    /// Returns the level of the music from 0 to 1: it follows the air and is silent in space and in warp.
    pub fn music_level(&self) -> f32 {
        if self.in_space || self.warping {
            return 0.0;
        }
        self.altitude_fade.clamp(0.0, 1.0)
    }
}

/// Returns how much air is left at `altitude` metres above sea level under an atmosphere `top`
/// metres high: 1 on the ground and below, 0 at the top and above, thinning faster near the ground.
pub fn altitude_fade(altitude: f32, top: f32) -> f32 {
    let left = 1.0 - (altitude / top.max(1.0)).clamp(0.0, 1.0);
    left * left
}
