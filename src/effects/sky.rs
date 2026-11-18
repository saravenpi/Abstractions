//! What the weather lets fall near the camera: rain streaks, snowflakes and blowing dust, out of
//! the same pool as every other particle, and nothing under a roof.

use super::kind::{Ambient, CEILING, Kind};
use crate::weather::Weather;
use crate::world::SEA_LEVEL;
use crate::world::structures::{FloorKind, Kind as Site};

/// Share of the pool rain takes in the hardest rain, and snow in the thickest snowfall.
const FALL_SHARE: f32 = 0.9;

/// Share of the pool dust takes in the worst dust storm.
const DUST_SHARE: f32 = 0.7;

/// What the weather drops around the camera, each from 0 to 1, and whether a roof is overhead.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Sky {
    /// How hard it rains.
    pub rain: f32,
    /// How hard it snows.
    pub snow: f32,
    /// How much dust the wind lifts.
    pub dust: f32,
    /// True under a roof, where nothing falls.
    pub sheltered: bool,
}

impl Sky {
    /// Returns what `weather` drops around a camera that is `sheltered` or not.
    pub fn of(weather: &Weather, sheltered: bool) -> Self {
        Self {
            rain: weather.rain,
            snow: weather.snow,
            dust: weather.dust,
            sheltered,
        }
    }

    /// Returns true when the weather drops anything here. When it does not, the particles are
    /// picked exactly as they were before there was weather.
    pub fn falling(&self) -> bool {
        !self.sheltered && self.rain + self.snow + self.dust > 0.0
    }
}

/// Returns true when the floor of a room of `floor` under the camera, with a site of `near`
/// the nearest around, is under a roof: the planks of a village house or a stilt hut, or the
/// floor of a cabin or a lighthouse. A platform, a plaza or a boardwalk is no room, see
/// [`crate::world::structures::Surroundings::room_at`].
pub fn roofed(floor: Option<FloorKind>, near: Option<Site>) -> bool {
    matches!(
        (floor, near),
        (Some(FloorKind::Wood), Some(Site::Village | Site::StiltHut))
            | (Some(_), Some(Site::Cabin | Site::Lighthouse))
    )
}

/// Returns the weather particle a roll in `[0, 1)` gives under `sky` in the surroundings
/// `ambient`, or nothing when the roll leaves the particle to the surroundings. Nothing falls
/// under a roof, under the sea or above every summit.
pub fn fall(sky: &Sky, ambient: &Ambient, roll: f32) -> Option<Kind> {
    let open = sky.falling() && ambient.height >= SEA_LEVEL && ambient.height <= CEILING;
    let shares = [
        (Kind::Rain, sky.rain * FALL_SHARE),
        (Kind::Snow, sky.snow * FALL_SHARE),
        (Kind::Dust, sky.dust * DUST_SHARE),
    ];
    let mut total = 0.0;
    shares
        .into_iter()
        .filter(|_| open)
        .find(|&(_, share)| {
            total += share;
            roll < total
        })
        .map(|(kind, _)| kind)
}

#[cfg(test)]
mod tests;
