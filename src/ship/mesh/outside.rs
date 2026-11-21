//! What is fixed to the outside of the hull: the heat shield plate under the belly, the service
//! panel and the kick steps of the left flank, the studs of the nose and the lights. Every part
//! is sunk into the skin.

use bevy::prelude::*;

use super::frame::rim;
use super::loft::{Section, solid};
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
    [1.5, 0.62, -0.2, -0.58],
    [2.2, 0.5, -0.1, -0.4],
    [2.7, 0.3, 0.0, -0.2],
];

/// Where the service panel is on the left flank of the nose along z, and how far up the flank
/// from the chine it starts and ends.
const PANEL: (f32, f32, (f32, f32)) = (-2.95, -2.5, (0.2, 0.75));

/// Where the two kick steps are on the left flank under the sill of the canopy along z, and how
/// far up the flank from the chine each starts and ends.
const STEPS: (f32, f32, [(f32, f32); 2]) = (-1.74, -1.46, [(0.14, 0.3), (0.5, 0.66)]);

/// Stations of the aerial on the spine, like the stations of a blade: height, z of the leading
/// edge and of the trailing edge, and half thickness. Its foot is inside the spine.
pub(super) const AERIAL: [[f32; 4]; 2] = [[1.0, -0.14, 0.2, 0.02], [1.38, 0.2, 0.36, 0.012]];

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
    let (edge, chine) = (rim(at, if across > 0.0 { 3 } else { 5 }), rim(at, 4));
    let normal = (edge - chine).cross(Vec3::Z).normalize();
    (
        chine.lerp(edge, across.abs()),
        normal * normal.x.signum() * -1.0,
    )
}

/// Adds one layer over the part of the flank `from..to` across and `front..back` along z: a box
/// sunk 2 cm in the skin that stands `lift` metres out of it.
pub(super) fn layer(
    solid: &mut Solid,
    (front, back): (f32, f32),
    (from, to): (f32, f32),
    lift: f32,
) {
    let corner = |z: f32, across: f32| {
        let (at, normal) = flank(z, across);
        (at - normal * 0.02, normal * (0.02 + lift))
    };
    let (a, b) = (corner(front, from), corner(back, to));
    let base = [a.0, corner(front, to).0, b.0, corner(back, from).0];
    solid.plate(base, (a.1 + b.1) * 0.5);
}

/// Returns the fittings of the left flank, which the right flank does not have: the service
/// panel of the nose, a slate frame with a dark cover, and the two dark kick steps under the sill
/// of the canopy.
pub fn left_side() -> Solid {
    let (front, back, across) = PANEL;
    let mut panel = Solid::default();
    panel.set_tone(1);
    layer(&mut panel, (front, back), across, 0.012);
    panel.set_tone(3);
    let cover = (across.0 + 0.06, across.1 - 0.06);
    layer(&mut panel, (front + 0.05, back - 0.05), cover, 0.02);
    panel.set_tone(4);
    for step in STEPS.2 {
        layer(&mut panel, (STEPS.0, STEPS.1), step, 0.008);
    }
    panel
}

/// Returns the rung under the left chine, by which the kick steps are reached from the ground:
/// two hangers out of the flank and a bar between them.
pub fn rung() -> Solid {
    let (front, back, _) = STEPS;
    let mut rung = Solid::default();
    for z in [front, back - 0.03] {
        rung.slab(Vec3::new(-0.8, -0.52, z), Vec3::new(-0.77, -0.02, z + 0.03));
    }
    rung.slab(
        Vec3::new(-0.86, -0.52, front),
        Vec3::new(-0.74, -0.49, back),
    );
    rung
}

/// Returns the studs: a row of three dark squares on each upper flank of the nose.
pub fn studs() -> Solid {
    let mut studs = Solid::default();
    studs.set_tone(3);
    for k in 0..3 {
        let front = -2.3 + 0.11 * k as f32;
        layer(&mut studs, (front, front + 0.06), (0.42, 0.52), 0.012);
    }
    studs.mirrored()
}

/// Returns the lights of the hull: the red lamp on the left wing tip, the green one on the
/// right, and the white ones, the tail light and the lens of the landing light.
pub fn lamps() -> [Solid; 3] {
    let lamp = |lo: Vec3, hi: Vec3| {
        let mut lamp = Solid::default();
        lamp.slab(lo, hi);
        lamp
    };
    let mut white = lamp(Vec3::new(-0.012, 1.9, 3.86), Vec3::new(0.012, 1.97, 3.98));
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
