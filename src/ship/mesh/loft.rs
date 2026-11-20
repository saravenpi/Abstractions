//! Lofting: closed shapes made of cross-sections. Neighbouring sections share their corner
//! points exactly, so the panels between them share edges and the shape is one watertight skin.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::ship::solid::Solid;

/// A cross-section: its corner points in order around the shape.
pub type Section = Vec<Vec3>;

/// Returns the middle of the corner points of a section.
pub fn centre(section: &[Vec3]) -> Vec3 {
    section.iter().sum::<Vec3>() / section.len() as f32
}

/// Returns a circle of `sides` corners, `radius` metres wide, around `at` in the plane of z.
pub fn round(at: Vec3, radius: f32, sides: u32) -> Section {
    let corner = |i: u32| {
        let a = i as f32 / sides as f32 * TAU;
        at + Vec3::new(a.cos() * radius, a.sin() * radius, 0.0)
    };
    (0..sides).map(corner).collect()
}

/// Returns a circle of `sides` corners, `radius` metres wide, around `at` in the plane of y.
pub fn flat(at: Vec3, radius: f32, sides: u32) -> Section {
    let corner = |i: u32| {
        let a = i as f32 / sides as f32 * TAU;
        at + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius)
    };
    (0..sides).map(corner).collect()
}

/// Adds the skin between each two neighbouring `sections`, facing away from the shape, or
/// into it with `inward`. The panel `i` between corners `i` and `i + 1` of the span `span`
/// takes the tone `tone(span, i)`.
pub fn skin(
    solid: &mut Solid,
    sections: &[Section],
    inward: bool,
    tone: impl Fn(usize, usize) -> u8,
) {
    let way = if inward { -1.0 } else { 1.0 };
    for (span, pair) in sections.windows(2).enumerate() {
        let (a, b) = (&pair[0], &pair[1]);
        let heart = (centre(a) + centre(b)) * 0.5;
        for i in 0..a.len() {
            let j = (i + 1) % a.len();
            let middle = (a[i] + a[j] + b[i] + b[j]) * 0.25;
            solid.set_tone(tone(span, i));
            solid.quad([a[i], a[j], b[j], b[i]], (middle - heart) * way);
        }
    }
}

/// Adds the fan that closes `section` with a centre point, facing `out`.
pub fn cap(solid: &mut Solid, section: &[Vec3], out: Vec3) {
    let heart = centre(section);
    for i in 0..section.len() {
        let j = (i + 1) % section.len();
        solid.tri([heart, section[i], section[j]], out);
    }
}

/// Adds the flat ring between the `outer` and the `inner` section, which have as many corners,
/// facing `out`.
pub fn annulus(solid: &mut Solid, outer: &[Vec3], inner: &[Vec3], out: Vec3) {
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        solid.quad([outer[i], outer[j], inner[j], inner[i]], out);
    }
}

/// Adds a closed shape: the skin between `sections`, and a fan over the first and the last,
/// which take the tone of the first panel of the first and of the last span.
pub fn solid(solid: &mut Solid, sections: &[Section], tone: impl Fn(usize, usize) -> u8) {
    let (first, last) = (&sections[0], &sections[sections.len() - 1]);
    let along = centre(first) - centre(&sections[1]);
    let behind = centre(last) - centre(&sections[sections.len() - 2]);
    let ends = (tone(0, 0), tone(sections.len() - 2, 0));
    skin(solid, sections, false, tone);
    solid.set_tone(ends.0);
    cap(solid, first, along);
    solid.set_tone(ends.1);
    cap(solid, last, behind);
}

/// Adds a closed body of revolution around the vertical through `at`: `rings` are the heights
/// and radii from the top down, and all of it takes `tone`.
pub fn column(part: &mut Solid, at: Vec3, rings: &[(f32, f32)], (sides, tone): (u32, u8)) {
    let sections: Vec<Section> = rings
        .iter()
        .map(|&(y, r)| flat(at + Vec3::Y * y, r, sides))
        .collect();
    solid(part, &sections, |_, _| tone);
}
