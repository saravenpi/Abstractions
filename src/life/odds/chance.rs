//! The chance that a planet lives: the share of every seed for which [`super::rung`] finds life.
//!
//! The age of the star is uniform over its lifetime `L` and the wait for life is exponential at
//! the rate `r`, so life has started with the chance `1 - (1 - exp(-r L)) / (r L)`.

use super::tune::{OLDEST, RATE};
use super::{World, speed, suitability};
use crate::universe::{GIANT_RADIUS, NO_AIR};

/// Returns true when `world` never lives whatever its liquids: a gas giant or an airless world.
pub(super) fn barren(world: &World) -> bool {
    world.radius >= GIANT_RADIUS.0 as f32 || world.pressure < NO_AIR as f32
}

/// Returns the lifetime of the star of `world` in billion years: about 10 at the home temperature
/// and shorter the hotter it is, as `(5780 / kelvin)^2.5`, and never more than [`OLDEST`].
pub(super) fn lifetime(world: &World) -> f32 {
    (10.0 * (5780.0 / world.star.max(1000.0)).powf(2.5)).min(OLDEST)
}

/// Returns the rate of abiogenesis on `world`, which suits life by `suits`, in events per billion
/// years.
pub(super) fn rate(world: &World, suits: f32) -> f32 {
    (RATE * suits * speed(world.temperature)).max(1e-9)
}

/// Returns the chance, 0 to 1, that `world` holds life, by the same odds [`super::rung`] rolls.
///
/// It draws nothing and costs a few float operations, so a card can show it for any planet.
pub fn chance(world: &World) -> f32 {
    if barren(world) {
        return 0.0;
    }
    let Some((_, suits)) = suitability(world) else {
        return 0.0;
    };
    let span = rate(world, suits) * lifetime(world);
    (1.0 + (-span).exp_m1() / span).clamp(0.0, 1.0)
}
