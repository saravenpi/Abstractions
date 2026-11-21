//! The jets of a ship that can raise dust, and where the flow of each meets the ground.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::super::Exhaust;
use super::tune::{GRAZE, REACH};
use crate::ship::mesh::{ENGINES, LIFT_AT, NOZZLE_Z};
use crate::ship::tune::{LIFT_ACCEL, MAIN_ACCEL, SHIP_MASS};
use crate::space::Placed;

/// Jets that can raise dust: the four lift nozzles, then the two main engines.
pub const JETS: usize = 6;

/// Thrust in newtons of one lift nozzle at its full push: a quarter of two and a half standard
/// gravities on the mass of the ship.
pub const NOZZLE: f32 = SHIP_MASS * LIFT_ACCEL as f32 / 4.0;

/// Thrust in newtons of one main engine at full throttle: half of three standard gravities on
/// the mass of the ship.
pub const ENGINE: f32 = SHIP_MASS * MAIN_ACCEL as f32 / 2.0;

/// One jet: where its exhaust leaves the nozzle, which way it flows and how hard it pushes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jet {
    /// Exit of the nozzle in the frame of the planet, in metres from its centre.
    pub from: DVec3,
    /// Unit direction the exhaust flows in, in frame axes.
    pub along: DVec3,
    /// Thrust in newtons.
    pub push: f32,
}

/// Where the flow of a jet meets the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    /// The point of the ground it meets, in the frame of the planet.
    pub at: DVec3,
    /// Metres the flow runs from the nozzle to it.
    pub run: f32,
    /// How hard it still blows there, from 1 right under the nozzle to 0 at the end of its reach.
    pub strength: f32,
}

/// Returns the jets of a ship placed at `placed` whose engines run as `exhaust` says: the lift
/// nozzles blow down along the floor of the hull, the main engines back along its tail.
pub fn jets(placed: &Placed, exhaust: &Exhaust) -> [Jet; JETS] {
    let at = |point: Vec3| placed.pos + placed.rot * point.as_dvec3();
    let (down, back) = (placed.rot * DVec3::NEG_Y, placed.rot * DVec3::Z);
    let belly = LIFT_AT.map(|point| Jet {
        from: at(point),
        along: down,
        push: NOZZLE * exhaust.lift,
    });
    let tail = ENGINES.map(|axis| Jet {
        from: at(axis.extend(NOZZLE_Z)),
        along: back,
        push: ENGINE * exhaust.main,
    });
    [belly[0], belly[1], belly[2], belly[3], tail[0], tail[1]]
}

/// Returns the distance in metres along a jet of `push` newtons within which its flow still
/// lifts dust: [`REACH`] for a lift nozzle at its full push, and the root of the thrust beyond.
pub fn reach(push: f32) -> f32 {
    REACH * (push.max(0.0) / NOZZLE).sqrt()
}

/// Returns where `jet` meets the ground within its reach. `floor` gives the distance from the
/// centre of the planet of the ground, or of the sea over it, in each unit direction. The jet is
/// followed from its nozzle down to the level of the ground under it, and the point it reaches
/// is put on the ground there. A jet that points up, grazes the ground or starts under it meets
/// nothing.
pub fn impinge(jet: &Jet, floor: impl Fn(DVec3) -> f64) -> Option<Hit> {
    let up = jet.from.normalize_or_zero();
    let fall = -jet.along.dot(up);
    let clear = jet.from.length() - floor(up);
    if fall < GRAZE || clear < 0.0 {
        return None;
    }
    let ahead = jet.from + jet.along * (clear / fall);
    let dir = ahead.normalize_or_zero();
    let at = dir * floor(dir);
    let (run, range) = (jet.from.distance(at) as f32, reach(jet.push));
    (run < range).then(|| Hit {
        at,
        run,
        strength: 1.0 - run / range,
    })
}
