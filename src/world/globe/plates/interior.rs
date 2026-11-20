//! The interior of a planet: how much of its heat is left, which decides whether its plates still move,
//! whether volcanoes grow and how many old impact basins it keeps.

use super::tune::{AGE_GYR, AIR_SHIELD, DRY_LID, EARTH_MASS, HEAT_GYR, LOWLAND, MOLTEN, SCAR_GYR};
use crate::universe::{GRAVITATION, NO_AIR, PlanetRecord, Water};
use crate::world::globe::roll::roll;
use crate::world::islands::smooth;

/// The grid cell the rolls of the interior are drawn at.
const KEY: [i64; 3] = [0x1A7E, 0, 0];

/// What the mass, the age, the water and the heat of a planet say about its crust.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interior {
    /// Share of the inner heat that is left, from 0 on a cold dead world to 1 on a young or molten one.
    pub activity: f32,
    /// How fast the plates move, from 0 to 1: the activity, less without liquid water.
    pub mobility: f32,
    /// Share of the surface that is low ground: the ocean share of the record, or a seeded share on a
    /// world without a sea.
    pub lowland: f64,
    /// Share of the impact basins the surface has kept, from 0 to 1.
    pub scars: f32,
}

impl Interior {
    /// Returns the interior of a planet of `radius` metres and surface `gravity` in metres per second
    /// squared, `age` billions of years old, with `record`.
    ///
    /// The heat lasts [`HEAT_GYR`] times the square root of the mass in Earth masses, and a surface hot
    /// enough to melt keeps the crust moving whatever the age. Old, dead, dry and airless worlds keep
    /// their basins.
    pub fn of(seed: u64, [radius, gravity]: [f64; 2], record: &PlanetRecord, age: f64) -> Self {
        let mass = gravity * radius * radius / GRAVITATION / EARTH_MASS;
        let left = (1.0 - age / (HEAT_GYR * mass.sqrt())).clamp(0.0, 1.0) as f32;
        let activity = left.max(smooth(MOLTEN.0, MOLTEN.1, record.temperature as f32));
        let lid = match record.water {
            Water::Liquid => 1.0,
            _ => DRY_LID,
        };
        let dry = 1.0 - record.ocean.clamp(0.0, 1.0) as f32;
        let bare = 1.0 - AIR_SHIELD * smooth(NO_AIR as f32, 1.0, record.pressure as f32);
        Self {
            activity,
            mobility: activity * lid,
            lowland: lowland(seed, record.ocean),
            scars: (age / SCAR_GYR).min(1.0) as f32 * (1.0 - activity) * dry * bare,
        }
    }
}

/// Returns the share of low ground of planet `seed` whose record gives it an `ocean` share.
fn lowland(seed: u64, ocean: f64) -> f64 {
    if ocean > 0.0 {
        return ocean.min(1.0);
    }
    LOWLAND.0 + (LOWLAND.1 - LOWLAND.0) * f64::from(roll(seed, KEY, 0))
}

/// Returns the age in billions of years of planet `seed` when nothing gave it one, spread evenly over
/// [`AGE_GYR`].
pub fn age(seed: u64) -> f64 {
    AGE_GYR.0 + (AGE_GYR.1 - AGE_GYR.0) * f64::from(roll(seed, KEY, 1))
}
