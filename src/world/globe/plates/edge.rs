//! Plate edges: what kind each one is by how its two plates move, and the shape of the ground across it.
//!
//! Every profile is a smooth function of the signed angle from the edge and is zero on the edge itself or
//! the same on both sides, so the ground never steps where the owner of a direction changes.

use super::Crust;
use super::tune::{ARC, CLOSING, CORDILLERA, FOSSIL, RANGE, RIDGE, RIDGE_RISE, RIFT, TRENCH};
use crate::world::islands::smooth;

/// What two plates do along their edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    /// The plates close in: ranges, trenches and island arcs.
    Convergent,
    /// The plates part: ocean ridges and rift valleys.
    Divergent,
    /// The plates slide past each other and leave the ground as it is.
    Transform,
}

impl Boundary {
    /// Returns the kind of an edge whose plates close in at `closing`, a share of the fastest plate that
    /// is negative when they part.
    pub fn of(closing: f64) -> Self {
        if closing > f64::from(CLOSING.0) {
            Boundary::Convergent
        } else if closing < -f64::from(CLOSING.0) {
            Boundary::Divergent
        } else {
            Boundary::Transform
        }
    }
}

/// What an edge adds to the ground at one place.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Uplift {
    /// Height added on land, as a share of the highest mountains. Negative in a rift.
    pub rise: f32,
    /// Height added to the sea floor, as a share of the ocean depth. Negative in a trench.
    pub floor: f32,
}

/// Two plates meeting, seen from one place near their edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Meet {
    /// Crust of the first and of the second plate.
    pub crust: [Crust; 2],
    /// Radians from the edge, positive on the side of the first plate.
    pub t: f32,
    /// How fast the plates close in there, as a share of the fastest plate. Negative when they part.
    pub closing: f32,
    /// How fast the plates of the planet move today, from 0 to 1.
    pub mobility: f32,
}

/// Returns what the edge of `meet` adds to the ground.
///
/// Two continents closing in raise a range on both sides, and parting open a rift. A sea plate sinks under
/// a continent, which gets a coastal range, and under the first of two sea plates, which gets an island
/// arc; the sinking side gets a trench. Two sea plates parting raise a ridge. A range outlives the motion
/// that built it at [`FOSSIL`] of its height; trenches, arcs, ridges and rifts need the plates to move.
pub fn profile(meet: &Meet) -> Uplift {
    let push = smooth(CLOSING.0, CLOSING.1, meet.closing);
    let pull = smooth(CLOSING.0, CLOSING.1, -meet.closing) * meet.mobility;
    let (old, live) = (
        push * (FOSSIL + (1.0 - FOSSIL) * meet.mobility),
        push * meet.mobility,
    );
    let t = meet.t / RANGE as f32;
    match meet.crust {
        [Crust::Continental, Crust::Continental] => Uplift {
            rise: old * bell(t) - pull * RIFT.0 * bell(t / RIFT.1),
            floor: 0.0,
        },
        [Crust::Oceanic, Crust::Oceanic] => Uplift {
            rise: 0.0,
            floor: live * (ARC * bump(t, 0.3) - TRENCH * bump(-t, 0.25)) + pull * ridge(meet.t),
        },
        [Crust::Continental, Crust::Oceanic] => margin(t, old, live),
        [Crust::Oceanic, Crust::Continental] => margin(-t, old, live),
    }
}

/// Returns what a sea plate sinking under a continent adds `t` ranges from their edge, positive on the
/// continent: a coastal range `old` strong inland and a trench `live` strong off the coast.
fn margin(t: f32, old: f32, live: f32) -> Uplift {
    Uplift {
        rise: old * CORDILLERA * bump(t, 0.5),
        floor: -live * TRENCH * bump(-t, 0.25),
    }
}

/// Returns a bell 1 high at 0 that reaches 0, with no slope left, at -1 and 1.
fn bell(x: f32) -> f32 {
    let k = (1.0 - x * x).max(0.0);
    k * k
}

/// Returns a bell 1 high at `centre` that reaches 0 at 0 and at twice `centre`.
fn bump(x: f32, centre: f32) -> f32 {
    bell((x - centre) / centre)
}

/// Returns the height of a spreading ridge `t` radians from its crest, as a share of the ocean depth: the
/// sea floor sinks as the square root of its age, so of its distance from the crest, down to the old floor
/// at [`RIDGE`].
fn ridge(t: f32) -> f32 {
    RIDGE_RISE * (1.0 - (t.abs() / RIDGE as f32).sqrt()).max(0.0)
}
