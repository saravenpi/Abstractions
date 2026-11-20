//! Stains: a drop of blood that lands leaves a mark on the ground, in the colour of the blood of the
//! species that bled.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::TRAIL;
use crate::effects::Landed;
use crate::fauna::place::east;
use crate::fauna::senses::{Blood as Marks, Drip};
use crate::life::blood::{drop_volume, stain};
use crate::settings::{Persistent, Settings};
use crate::space::{Anchor, Clock};
use crate::world::Terrain;

/// Returns the drop that a fleck of `radius` metres leaves where it landed at unit direction `at`
/// flying along `heading` in planet axes, species `species`, at universe time `time`: a stain as wide
/// as the film the drop spreads into, kept within [`TRAIL`].
pub fn drip(at: DVec3, heading: DVec3, (species, radius): (u16, f32), time: f64) -> Drip {
    let flat = heading - at * heading.dot(at);
    let facing = if flat.length_squared() > 1e-12 {
        flat.normalize()
    } else {
        east(at)
    };
    Drip {
        at,
        facing,
        species,
        size: stain(drop_volume(radius)).clamp(TRAIL[0], TRAIL[1]),
        time,
    }
}

/// Lays a mark of every drop of blood that landed, unless the blood is turned off.
pub fn lay(
    mut landed: MessageReader<Landed>,
    (anchor, terrain, clock): (Res<Anchor>, Res<Terrain>, Res<Clock>),
    settings: Res<Persistent<Settings>>,
    mut marks: ResMut<Marks>,
) {
    if !settings.gameplay.blood {
        landed.clear();
        return;
    }
    let radius = terrain.globe().radius();
    for drop in landed.read() {
        let Some(species) = drop.tag else { continue };
        let at = anchor.to_frame(drop.at).normalize();
        let heading = anchor.rotation.inverse() * drop.heading.as_dvec3();
        marks.bleed(
            drip(at, heading, (species, drop.radius), clock.seconds),
            None,
            radius,
        );
    }
}
