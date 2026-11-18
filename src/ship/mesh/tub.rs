//! The cockpit tub: the floor, the side walls, the front wall and the rear bulkhead that hang
//! from the lips of the mouth in the hull, facing inward.

use bevy::prelude::*;

use super::frame::{SIDES, ring};
use super::shell::{HALF, lip, mouth};
use crate::ship::cockpit::FLOOR;
use crate::ship::solid::Solid;

/// Returns the point of the floor under the lip of `station` that is `half` metres from the middle.
fn foot(station: [f32; 4], half: f32) -> Vec3 {
    Vec3::new(half, FLOOR, station[0])
}

/// Adds the wall that closes the tub across the hull at `station`, from the roof line down to
/// the floor, facing `out`.
fn wall(solid: &mut Solid, station: [f32; 4], half: f32, out: Vec3) {
    let ridge = ring(station, 2, SIDES);
    let (left, right) = (lip(station, -half), lip(station, half));
    let (low_left, low_right) = (foot(station, -half), foot(station, half));
    solid.tri([ridge, right, low_right], out);
    solid.tri([ridge, low_right, low_left], out);
    solid.tri([ridge, low_left, left], out);
}

/// Returns the tub. Its rim is the lip of the mouth, so no gap shows between the two.
pub fn tub() -> Solid {
    let mut tub = Solid::default();
    let stations = mouth();
    for i in 0..2 {
        let (a, b, wa, wb) = (stations[i], stations[i + 1], HALF[i], HALF[i + 1]);
        for side in [-1.0, 1.0] {
            let rim = [lip(a, wa * side), lip(b, wb * side)];
            let low = [foot(b, wb * side), foot(a, wa * side)];
            tub.quad([rim[0], rim[1], low[0], low[1]], Vec3::X * -side);
        }
        let floor = [foot(a, -wa), foot(a, wa), foot(b, wb), foot(b, -wb)];
        tub.quad(floor, Vec3::Y);
    }
    wall(&mut tub, stations[0], HALF[0], Vec3::Z);
    wall(&mut tub, stations[2], HALF[2], Vec3::NEG_Z);
    tub
}
