//! What is bolted into the cockpit tub, in the plate-over-frame language of the hull: the dark
//! frame of the seat, of the side consoles and of the dashboard, the ribs along the walls, and
//! over them the light plates, the pads of the seat, the tops of the consoles, the bezels of the
//! displays, the cable runs and the heel plate. The harness is the accent.
//!
//! The ribs follow the lips of the mouth and the walls of the tub, which `shell` and `tub`
//! own, and the rest stands on the places `cockpit::tune` gives: nothing here knows the size
//! of the hull.

mod consoles;
mod dash;
mod ribs;
mod seat;

use bevy::prelude::*;

use super::loft::{Section, solid};
use crate::ship::solid::Solid;

/// Adds a box between `lo` and `hi` whose four edges along z are cut by `cut` metres, in `tone`.
pub(super) fn beam(part: &mut Solid, (lo, hi): (Vec3, Vec3), cut: f32, tone: u8) {
    let ring = |z: f32| -> Section {
        [
            (lo.x + cut, hi.y),
            (hi.x - cut, hi.y),
            (hi.x, hi.y - cut),
            (hi.x, lo.y + cut),
            (hi.x - cut, lo.y),
            (lo.x + cut, lo.y),
            (lo.x, lo.y + cut),
            (lo.x, hi.y - cut),
        ]
        .map(|(x, y)| Vec3::new(x, y, z))
        .into()
    };
    solid(part, &[ring(lo.z), ring(hi.z)], |_, _| tone);
}

/// Returns the dark frame of the cabin: the shell and the foot of the seat, the side consoles,
/// the ribs along the walls, and the dashboard with its hood.
pub fn furniture() -> Solid {
    let mut dark = seat::shell();
    for part in [consoles::frames(), dash::dash(), ribs::ribs()] {
        dark.merge(&part, Vec3::ZERO);
    }
    dark
}

/// Returns the light plates of the cabin, which take the skin of the hull: the pads of the seat,
/// the tops of the consoles with their cable runs, the bezels of the displays and the heel plate.
pub fn plates() -> Solid {
    let mut light = seat::pads();
    for part in [consoles::tops(), dash::bezels()] {
        light.merge(&part, Vec3::ZERO);
    }
    light
}

/// Returns the accents of the cabin: the harness of the seat.
pub fn accents() -> Solid {
    seat::harness()
}

#[cfg(test)]
mod tests_dash;
