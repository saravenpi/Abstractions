//! What is fixed to the outside of the hull: the heat shield plate under the belly, the boarding
//! hatch on the left flank, the antenna mast and the lights. Every part is sunk into the skin.

use bevy::prelude::*;

use super::frame::ring;
use super::loft::{Section, column, solid};
use super::shell::station;
use crate::ship::solid::Solid;

/// Stations of the heat shield plate from the nose: z, half width, height of the top and of the
/// bottom. The top is inside the hull. The bottom is 0.65 m under the middle of the ship, the
/// lowest point of the hull, and the plate narrows to nothing at both ends.
pub(super) const SHIELD: [[f32; 4]; 7] = [
    [-3.0, 0.1, -0.25, -0.3],
    [-2.2, 0.3, -0.25, -0.5],
    [-1.0, 0.5, -0.25, -0.62],
    [0.4, 0.62, -0.25, -0.65],
    [1.5, 0.62, -0.25, -0.65],
    [2.2, 0.55, -0.25, -0.58],
    [2.7, 0.3, -0.2, -0.34],
];

/// Where the hatch is on the left flank along z, and how far across the flank its door reaches,
/// from the chine up and down.
const HATCH: (f32, f32, f32) = (-2.25, -1.45, 0.95);

/// Returns the section of the heat shield at a station: a box with chamfered edges.
fn box_section([z, half, top, bottom]: [f32; 4]) -> Section {
    let c = (half * 0.4).min(0.12);
    let at = |x: f32, y: f32| Vec3::new(x, y, z);
    vec![
        at(-half + c, top),
        at(half - c, top),
        at(half, top - c),
        at(half, bottom + c),
        at(half - c, bottom),
        at(-half + c, bottom),
        at(-half, bottom + c),
        at(-half, top - c),
    ]
}

/// Returns the heat shield: the charcoal plate that makes the underside of the hull.
pub fn shield() -> Solid {
    let sections: Vec<Section> = SHIELD.iter().map(|&at| box_section(at)).collect();
    let mut plate = Solid::default();
    solid(&mut plate, &sections, |_, _| 3);
    plate
}

/// Returns the point of the left flank at `z`, `across` of the way from the chine up toward the
/// roof for a positive share and down toward the belly for a negative one, and the unit normal
/// of the flank there.
fn flank(z: f32, across: f32) -> (Vec3, Vec3) {
    let at = station(z);
    let (edge, chine) = (
        ring(at, if across > 0.0 { 3 } else { 5 }, 8),
        ring(at, 4, 8),
    );
    let normal = (edge - chine).cross(Vec3::Z).normalize();
    (
        chine.lerp(edge, across.abs()),
        normal * normal.x.signum() * -1.0,
    )
}

/// Adds one layer of the hatch over the part of the flank `from..to` across and `front..back`
/// along z: a box sunk 2 cm in the skin that stands `lift` metres out of it.
fn layer(solid: &mut Solid, (front, back): (f32, f32), (from, to): (f32, f32), lift: f32) {
    let corner = |z: f32, across: f32| {
        let (at, normal) = flank(z, across);
        (at - normal * 0.02, normal * (0.02 + lift))
    };
    let (a, b) = (corner(front, from), corner(back, to));
    let base = [a.0, corner(front, to).0, b.0, corner(back, from).0];
    solid.plate(base, (a.1 + b.1) * 0.5);
}

/// Returns the boarding hatch on the left flank, where the robot steps out: a light frame, the
/// dark door in it, and the handle, over the upper and the lower face of the flank.
pub fn hatch() -> Solid {
    let (front, back, reach) = HATCH;
    let mut hatch = Solid::default();
    for (from, to) in [(0.0, reach), (-reach, 0.0)] {
        hatch.set_tone(1);
        layer(&mut hatch, (front, back), (from, to), 0.03);
        hatch.set_tone(4);
        let (inner, scale) = ((front + 0.1, back - 0.1), 0.88);
        layer(&mut hatch, inner, (from * scale, to * scale), 0.05);
    }
    hatch.set_tone(1);
    let chine = flank(-1.65, 0.0).0;
    hatch.slab(
        chine + Vec3::new(-0.08, -0.04, -0.05),
        chine + Vec3::new(0.0, 0.06, 0.05),
    );
    hatch
}

/// Returns the antenna: a mast on the spine behind the canopy with a crossbar at its top.
pub fn mast() -> Solid {
    let mut mast = Solid::default();
    column(
        &mut mast,
        Vec3::new(0.0, 0.0, 0.95),
        &[(1.12, 0.015), (0.7, 0.025)],
        (6, 0),
    );
    mast.slab(Vec3::new(-0.1, 1.1, 0.93), Vec3::new(0.1, 1.13, 0.97));
    mast
}

/// Returns the lights of the hull: the red lamp on the left wing tip, the green one on the
/// right, and the white ones, the tail light and the lens of the landing light.
pub fn lamps() -> [Solid; 3] {
    let lamp = |lo: Vec3, hi: Vec3| {
        let mut lamp = Solid::default();
        lamp.slab(lo, hi);
        lamp
    };
    let mut white = lamp(Vec3::new(-0.012, 1.7, 3.33), Vec3::new(0.012, 1.78, 3.43));
    white.merge(
        &lamp(
            Vec3::new(-0.05, -0.35, -3.42),
            Vec3::new(0.05, -0.25, -3.32),
        ),
        Vec3::ZERO,
    );
    [
        lamp(Vec3::new(-3.3, 0.12, 2.05), Vec3::new(-3.2, 0.2, 2.18)),
        lamp(Vec3::new(3.2, 0.12, 2.05), Vec3::new(3.3, 0.2, 2.18)),
        white,
    ]
}
