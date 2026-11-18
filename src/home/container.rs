//! The container the operator never leaves: a sealed box of light metal on four legs.

use bevy::math::Affine3A;
use bevy::prelude::*;

use super::draft::Draft;
use super::glow::Glow;
use super::lot::Lot;
use super::paint::{BEVEL, DARK, HATCH, HULL, TRIM};
use super::soup::{Rgb, Soup};
use super::tune::CONTAINER;

/// Length, height and depth of the box in metres.
pub const SIZE: Vec3 = Vec3::new(6.0, 2.8, 2.6);

/// Where the legs stand, right and back of the centre of the box.
pub const FEET: [Vec2; 4] = [
    Vec2::new(-2.5, -1.0),
    Vec2::new(2.5, -1.0),
    Vec2::new(2.5, 1.0),
    Vec2::new(-2.5, 1.0),
];

/// Height in metres of the underside of the box above the highest ground under its legs.
pub const CLEAR: f32 = 0.4;

/// Centre of the hatch, on the front wall.
pub const DOOR: Vec3 = Vec3::new(-0.9, 1.3, 1.36);

/// Where the cables leave the box, on the end wall that faces the station.
pub const SOCKET: Vec3 = Vec3::new(-3.0, 0.35, 0.0);

/// The mast and the whip of the antenna cluster: where each stands on the roof, right and back
/// of the centre, its height and its half width in metres.
const RODS: [(Vec2, f32, f32); 2] = [
    (Vec2::new(1.7, -0.7), 1.5, 0.035),
    (Vec2::new(2.25, -0.8), 0.7, 0.015),
];

/// Width in metres of the bevel along every long edge.
const CHAMFER: f32 = 0.18;

/// Returns the pose of the container on `lot`, in the frame of the base.
pub fn pose(lot: &Lot) -> Affine3A {
    lot.stand(CONTAINER, &FEET, CLEAR)
}

/// Adds a bevelled box section from `x.0` to `x.1`, `grow` metres larger than the box all
/// round, closed at both ends, with the box around it as its body. The flat sides take the first
/// colour and the bevels the second.
fn hull(soup: &mut Soup, x: (f32, f32), grow: f32, rgb: [Rgb; 2]) {
    let (w, lo, hi, c) = (SIZE.z * 0.5 + grow, -grow, SIZE.y + grow, CHAMFER);
    let outline = [
        (c - w, lo),
        (w - c, lo),
        (w, lo + c),
        (w, hi - c),
        (w - c, hi),
        (c - w, hi),
        (-w, hi - c),
        (-w, lo + c),
    ];
    let ring = |x: f32, i: usize| Vec3::new(x, outline[i % 8].1, outline[i % 8].0);
    let centre = Vec3::new(0.0, SIZE.y * 0.5, 0.0);
    for i in 0..8 {
        let side = [
            ring(x.0, i),
            ring(x.1, i),
            ring(x.1, i + 1),
            ring(x.0, i + 1),
        ];
        let out = (side[0] + side[2]) * 0.5 - centre;
        soup.quad(side, out * Vec3::new(0.0, 1.0, 1.0), rgb[i % 2]);
    }
    for (end, out) in [(x.0, Vec3::NEG_X), (x.1, Vec3::X)] {
        for i in 1..7 {
            soup.tri([ring(end, 0), ring(end, i), ring(end, i + 1)], out, rgb[0]);
        }
    }
    soup.body(Vec3::new(x.0, lo, -w), Vec3::new(x.1, hi, w));
}

/// Adds the box, its end frames, its legs and the roof vent.
fn shell(draft: &mut Draft) {
    let half = SIZE.x * 0.5;
    draft.draw(|soup| {
        hull(soup, (-half, half), 0.0, [HULL, BEVEL]);
        hull(soup, (-half - 0.04, -half + 0.3), 0.04, [TRIM, TRIM]);
        hull(soup, (half - 0.3, half + 0.04), 0.04, [TRIM, TRIM]);
    });
    for foot in FEET {
        draft.leg(foot, 0.05, 0.09, DARK);
    }
    let top = SIZE.y;
    draft.deck(
        Vec3::new(-1.6, top, -0.5),
        Vec3::new(-0.7, top + 0.12, 0.3),
        TRIM,
    );
}

/// Adds the sealed hatch with its seam, its lock plate and the status strip above it.
fn hatch(draft: &mut Draft) {
    let (x, z) = (DOOR.x, SIZE.z * 0.5);
    draft.draw(|soup| {
        soup.slab(
            Vec3::new(x - 0.63, 0.22, z - 0.02),
            Vec3::new(x + 0.63, 2.38, z + 0.03),
            DARK,
        );
        soup.slab(
            Vec3::new(x - 0.55, 0.3, z),
            Vec3::new(x + 0.55, 2.3, z + 0.06),
            HATCH,
        );
        soup.slab(
            Vec3::new(x - 0.09, 1.2, z + 0.06),
            Vec3::new(x + 0.09, 1.4, z + 0.08),
            DARK,
        );
    });
    let strip = (
        Vec3::new(x - 0.4, 2.44, z),
        Vec3::new(x + 0.4, 2.49, z + 0.035),
    );
    draft.light(Glow::Strip, strip.0, strip.1);
}

/// Adds the narrow tinted window that glows from inside, and the faint light it casts.
fn window(draft: &mut Draft) {
    let z = SIZE.z * 0.5;
    draft.draw(|soup| {
        soup.slab(
            Vec3::new(0.62, 1.64, z - 0.02),
            Vec3::new(2.38, 1.94, z + 0.015),
            DARK,
        );
    });
    draft.light(
        Glow::Window,
        Vec3::new(0.7, 1.72, z),
        Vec3::new(2.3, 1.86, z + 0.03),
    );
    draft.lamp(Glow::Window, Vec3::new(1.5, 1.7, z + 0.7));
}

/// Adds a six-sided dish of `radius` centred on `centre`, open along `axis`, with its feed
/// and the strut that holds it from `foot`, and the box around the dish as its body.
fn dish(soup: &mut Soup, centre: Vec3, axis: Vec3, foot: Vec3) {
    let (radius, side) = (0.38, axis.any_orthonormal_vector());
    let rim = |i: u32| {
        let turn = Quat::from_axis_angle(axis, i as f32 * std::f32::consts::TAU / 6.0);
        centre + turn * side * radius
    };
    for i in 0..6 {
        let edge = [rim(i), rim(i + 1)];
        let out = (edge[0] + edge[1]) * 0.5 - centre - axis * radius;
        soup.tri([centre - axis * 0.16, edge[0], edge[1]], out, TRIM);
        soup.tri([centre - axis * 0.07, edge[0], edge[1]], axis, HATCH);
    }
    soup.beam(foot, centre - axis * 0.14, 0.03, DARK);
    soup.beam(centre - axis * 0.07, centre + axis * 0.22, 0.012, DARK);
    soup.body(centre - Vec3::splat(radius), centre + Vec3::splat(radius));
}

/// Adds the antenna cluster of the quantum link on the roof: a mast with its pulsing light, a
/// whip and a dish.
fn antenna(draft: &mut Draft) {
    let roof = |x: f32, z: f32, up: f32| Vec3::new(x, SIZE.y + up, z);
    let aim = Vec3::new(0.3, 0.8, 0.52).normalize();
    draft.draw(|soup| {
        soup.slab(roof(1.5, -0.95, 0.0), roof(2.4, -0.1, 0.06), TRIM);
        for (at, height, half) in RODS {
            soup.beam(roof(at.x, at.y, 0.0), roof(at.x, at.y, height), half, DARK);
        }
        dish(soup, roof(2.1, -0.45, 0.5), aim, roof(2.1, -0.45, 0.0));
    });
    let tip = roof(RODS[0].0.x, RODS[0].0.y, RODS[0].1 + 0.06);
    let half = Vec3::splat(0.06);
    draft.light(Glow::Pulse, tip - half, tip + half);
}

/// Draws the container at its spot.
pub fn build(draft: &mut Draft) {
    draft.stand(CONTAINER, &FEET, CLEAR);
    shell(draft);
    hatch(draft);
    window(draft);
    antenna(draft);
}
