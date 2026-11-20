//! Rings: a flat disc of ice or dust around a planet, always inside its Roche limit.

use bevy::math::Vec3;

use super::hash::{self, Rng};
use super::record::PlanetClass;
use super::spec::{PlanetSpec, Rings};

/// Closest and farthest inner edge of a ring disc, in radii of its planet.
pub const RING_INNER: (f64, f64) = (1.2, 1.6);

/// Farthest outer edge of a ring disc, in radii of its planet.
pub const RING_OUTER: f64 = 2.4;

/// The Roche limit in radii of the planet, inside which tides tear a loose moon into a ring.
///
/// This is the fluid limit, 2.44 radii, for ring matter as dense as the planet. Lighter ice
/// only moves the real limit out, so a disc inside this one is inside the real one too.
pub const ROCHE: f64 = 2.44;

/// Least and most share of the light a ring disc stops.
pub const RING_DENSITY: (f32, f32) = (0.2, 0.9);

/// Narrowest ring disc, in radii of its planet.
const WIDTH: f64 = 0.3;

/// Share of gas giants and of solid planets that have rings: a solid planet rarely holds enough
/// loose matter inside its Roche limit.
const ODDS: (f64, f64) = (0.4, 0.03);

/// Kelvin under which a planet is cold enough for ring ice to last.
const ICE_KELVIN: f64 = 220.0;

/// Linear RGB colour of rings of ice, pale and slightly warm.
const ICY: Vec3 = Vec3::new(0.80, 0.78, 0.72);

/// Linear RGB colour of rings of dust, a dark brown grey.
const DUSTY: Vec3 = Vec3::new(0.42, 0.34, 0.26);

/// Child of the planet seed the ring draws come from.
const RING_STREAM: u64 = 4;

/// Returns the rings of a planet, or nothing.
///
/// Four in ten gas giants and three in a hundred solid planets have rings. The inner edge
/// lies 1.2 to 1.6 radii out, the outer one at least 0.3 radii further and at most 2.4 radii
/// out, inside the [`ROCHE`] limit and so inside every moon, which orbit outside it. A disc
/// whose inner edge would lie in the air is not drawn. Rings are ice under 220 K and dust above.
pub(super) fn of(planet: &PlanetSpec) -> Option<Rings> {
    let mut rng = Rng::new(hash::child(planet.seed, RING_STREAM));
    let giant = planet.record.class == PlanetClass::GasGiant;
    let odds = if giant { ODDS.0 } else { ODDS.1 };
    let drawn = rng.unit() < odds;
    let inner = rng.range(RING_INNER.0, RING_INNER.1);
    let outer = rng.range(inner + WIDTH, RING_OUTER);
    let density = RING_DENSITY.0 + (RING_DENSITY.1 - RING_DENSITY.0) * rng.unit() as f32;
    let clear = (inner - 1.0) * planet.radius > planet.atmosphere_height;
    let icy = planet.record.temperature < ICE_KELVIN;
    (drawn && clear).then_some(Rings {
        inner: inner * planet.radius,
        outer: outer * planet.radius,
        density,
        color: if icy { ICY } else { DUSTY },
    })
}
