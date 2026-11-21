//! Fair places: where the star climbs high over a day and the mean air is mild enough for plants,
//! which is what a home base needs of the ground it stands on.

use bevy::math::DVec3;

use super::biome::HOT;
use super::{MEAN_SEASON, Setting, Terms, terms};
use crate::universe::{Daystar, LIT, MILD};

/// What a place whose mean air is not mild lacks before any light is counted: more than the most
/// light a place can lack, so that a mild place in the dark still comes before it.
const HARSH: f32 = 2.0;

/// The light and the warmth of one planet, for asking how fair its places are.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fair {
    /// The terms of the annual mean air temperature.
    terms: Terms,
    /// Where the star stands over one day.
    star: Daystar,
    /// Sine of the height [`LIT`] the star must reach.
    lit: f64,
}

impl Fair {
    /// Returns the light and the warmth of the planet of `setting`, whose star goes the way of
    /// `star` over a day.
    pub fn of(setting: &Setting, star: Daystar) -> Self {
        Self {
            terms: terms(setting, MEAN_SEASON),
            star,
            lit: LIT.to_radians().sin(),
        }
    }

    /// Returns what the place in unit direction `dir` lacks to be fair, 0 when it lacks nothing:
    /// over a day the star reaches [`LIT`] there, and the annual mean air at sea level is between
    /// the cold end of [`MILD`] and the heat over which dry land turns to desert, 299 K, so that
    /// plants stand in the light. A mild place the star stays low over lacks the sine it stays
    /// short by, at most 1.5. A place that is not mild lacks [`HARSH`] more. The same rule reads
    /// a planet that turns and one locked to its star: only where the star goes differs.
    pub fn lack(&self, dir: DVec3) -> f32 {
        let air = self.terms.at(dir.y as f32, dir.as_vec3(), 0.0);
        let dim = (self.lit - self.star.peak(dir)).max(0.0) as f32;
        match (MILD.0 as f32..=HOT).contains(&air) {
            true => dim,
            false => HARSH + dim,
        }
    }
}
