//! The shapes of the signs: each one a few flat coloured quads on the ground, faded as it ages.

use bevy::math::Vec2;

use crate::life::signs::{Mark, Sign};

/// One flat quad on the ground of the frame of the signs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    /// Centre over the ground, `(x, z)` in the frame.
    pub centre: Vec2,
    /// Unit direction of its length, `(x, z)` in the frame.
    pub along: Vec2,
    /// Half its length and half its width in metres.
    pub half: Vec2,
    /// Linear colour and opacity.
    pub colour: [f32; 4],
}

/// Half length and half width of a footprint, in lengths of the print.
const FOOT: Vec2 = Vec2::new(0.5, 0.32);

/// Place and half size of the soil dug out of a burrow, in widths of its mouth.
const SPOIL: (Vec2, Vec2) = (Vec2::new(0.45, 0.0), Vec2::new(0.55, 0.7));

/// Colour of a footprint: pressed, darker ground.
const PRESSED: [f32; 3] = [0.05, 0.04, 0.03];

/// Colour of a dropping.
const DUNG: [f32; 3] = [0.11, 0.07, 0.035];

/// Colour of cropped, drying plants.
const CROPPED: [f32; 3] = [0.5, 0.45, 0.26];

/// Colour of the gathered stems of a nest.
const STEMS: [f32; 3] = [0.28, 0.19, 0.09];

/// Colour of a burrow mouth and of the soil dug out of it.
const HOLE: ([f32; 3], [f32; 3]) = ([0.015, 0.012, 0.01], [0.32, 0.25, 0.17]);

/// Colour of weathered bone.
const BONE: [f32; 3] = [0.82, 0.79, 0.7];

/// Returns `rgb` at opacity `alpha`.
fn tint(rgb: [f32; 3], alpha: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], alpha.clamp(0.0, 1.0)]
}

/// Calls `put` with the quads of `sign`, lying at `centre` and pointing along the unit `along`, both
/// `(x, z)` in the frame.
pub fn quads(sign: &Sign, (centre, along): (Vec2, Vec2), put: &mut impl FnMut(Quad)) {
    let (s, f) = (sign.size, sign.fresh);
    let across = along.perp();
    let diagonal = (along + across).normalize_or(along);
    let mut at = |offset: Vec2, way: Vec2, half: Vec2, colour| {
        put(Quad {
            centre: centre + (along * offset.x + across * offset.y) * s,
            along: way,
            half: half * s,
            colour,
        });
    };
    match sign.mark {
        Mark::Print => at(Vec2::ZERO, along, FOOT, tint(PRESSED, 0.55 * f)),
        Mark::Dropping => crossed(&mut at, (along, diagonal), tint(DUNG, 0.95 * f)),
        Mark::Grazed => grazed(&mut at, (along, diagonal), f),
        Mark::Nest => nest(&mut at, (along, across)),
        Mark::Burrow => {
            at(SPOIL.0, along, SPOIL.1, tint(HOLE.1, 0.7));
            crossed(&mut at, (along, diagonal), tint(HOLE.0, 0.95));
        }
        Mark::Bones => bones(&mut at, (along, across), f),
    }
}

/// Puts two crossed quads of `colour` through `at`, a rough round blot.
fn crossed(
    at: &mut impl FnMut(Vec2, Vec2, Vec2, [f32; 4]),
    (along, diagonal): (Vec2, Vec2),
    colour: [f32; 4],
) {
    at(Vec2::ZERO, along, Vec2::splat(0.5), colour);
    at(Vec2::ZERO, diagonal, Vec2::splat(0.45), colour);
}

/// Cropped spots in a grazed patch, on a sunflower spiral.
const CROPS: usize = 16;

/// Puts the quads of a grazed patch through `at`, as pale as it is `fresh`: cropped spots on a
/// sunflower spiral, each small enough to follow the ground.
fn grazed(at: &mut impl FnMut(Vec2, Vec2, Vec2, [f32; 4]), ways: (Vec2, Vec2), fresh: f32) {
    let colour = tint(CROPPED, 0.35 * fresh);
    for k in 0..CROPS {
        let rank = k as f32 + 0.5;
        let reach = 0.45 * (rank / CROPS as f32).sqrt();
        let way = if k % 2 == 0 { ways.0 } else { ways.1 };
        at(
            Vec2::from_angle(rank * 2.4) * reach,
            way,
            Vec2::splat(0.07),
            colour,
        );
    }
}

/// Puts the quads of a nest through `at`: a ring of six stems around a dark hollow.
fn nest(at: &mut impl FnMut(Vec2, Vec2, Vec2, [f32; 4]), (along, across): (Vec2, Vec2)) {
    at(Vec2::ZERO, along, Vec2::splat(0.3), tint(HOLE.0, 0.6));
    for k in 0..6 {
        let angle = k as f32 * std::f32::consts::TAU / 6.0;
        let (sin, cos) = angle.sin_cos();
        let way = (along * -sin + across * cos).normalize_or(along);
        at(
            Vec2::new(cos, sin) * 0.36,
            way,
            Vec2::new(0.24, 0.05),
            tint(STEMS, 0.95),
        );
    }
}

/// Puts the quads of a skeleton through `at`, as white as it is `fresh`: a spine, a skull ahead of
/// it, three ribs across it and two leg bones behind.
fn bones(
    at: &mut impl FnMut(Vec2, Vec2, Vec2, [f32; 4]),
    (along, across): (Vec2, Vec2),
    fresh: f32,
) {
    let white = tint(BONE, 0.25 + 0.7 * fresh);
    at(Vec2::ZERO, along, Vec2::new(0.38, 0.025), white);
    at(Vec2::new(0.46, 0.0), along, Vec2::new(0.09, 0.06), white);
    for k in 0..3 {
        let x = 0.15 - 0.1 * k as f32;
        at(Vec2::new(x, 0.0), across, Vec2::new(0.13, 0.015), white);
    }
    for side in [-1.0, 1.0] {
        let way = (along + across * side * 0.4).normalize_or(along);
        at(
            Vec2::new(-0.42, 0.06 * side),
            way,
            Vec2::new(0.16, 0.02),
            white,
        );
    }
}
