//! The loft: a closed shape through cross-sections stacked along the Y axis.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::Rgb;
use super::soup::Soup;

/// One cross-section of a loft: its height, the middle of its ring and the half extents of the ring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Section {
    /// Height of the ring.
    pub y: f32,
    /// Middle of the ring in X and Z.
    pub at: Vec2,
    /// Half width in X and half depth in Z.
    pub half: Vec2,
}

impl Section {
    /// Returns a section at height `y` centred on the axis, `x` wide and `z` deep, half extents.
    pub const fn new(y: f32, x: f32, z: f32) -> Self {
        Self {
            y,
            at: Vec2::ZERO,
            half: Vec2::new(x, z),
        }
    }

    /// Returns the section moved by `z`, toward the back when positive.
    pub const fn back(mut self, z: f32) -> Self {
        self.at = Vec2::new(self.at.x, self.at.y + z);
        self
    }

    /// Returns the point of the ring at `x` and `z`, each from -1 to 1 across the section.
    fn point(&self, x: f32, z: f32) -> Vec3 {
        Vec3::new(
            self.at.x + x * self.half.x,
            self.y,
            self.at.y + z * self.half.y,
        )
    }
}

/// The outline every ring of a loft has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ring {
    /// A superellipse of this many corners and this power: 2 is round, more is squarer.
    Round(u32, f32),
    /// A rectangle whose corners are cut by this many meters. Zero leaves the rectangle.
    Cut(f32),
}

impl Ring {
    /// Returns the corners of the ring of `section`, the same number and order for every section.
    fn corners(self, section: &Section) -> Vec<Vec3> {
        match self {
            Ring::Round(sides, power) => {
                let curve = |t: f32| t.signum() * t.abs().powf(2.0 / power);
                let corner = |i: u32| {
                    let angle = i as f32 / sides as f32 * TAU;
                    section.point(curve(angle.cos()), curve(angle.sin()))
                };
                (0..sides).map(corner).collect()
            }
            Ring::Cut(cut) => {
                let cut = cut.min(section.half.min_element() * 0.8);
                let (x, z) = (1.0 - cut / section.half.x, 1.0 - cut / section.half.y);
                let all = [(1.0, -z), (1.0, z), (x, 1.0), (-x, 1.0)];
                let all = all.into_iter().chain(all.map(|(x, z)| (-x, -z)));
                let step = if cut > 0.0 { 1 } else { 2 };
                all.step_by(step).map(|(x, z)| section.point(x, z)).collect()
            }
        }
    }
}

/// Adds the lid of a loft: a fan over `ring` facing `out`, in the colour `rgb`.
fn lid(soup: &mut Soup, ring: &[Vec3], out: Vec3, rgb: Rgb) {
    let heart = ring.iter().sum::<Vec3>() / ring.len() as f32;
    for (i, corner) in ring.iter().enumerate() {
        soup.tri([heart, *corner, ring[(i + 1) % ring.len()]], out, rgb);
    }
}

/// Adds a closed shape through `sections`, ordered along Y, each ring of the outline `ring`.
/// A side of the span `span` that faces `out` takes the colour `tone(span, out)`; a lid takes the
/// colour of its span, facing up or down.
pub fn loft(soup: &mut Soup, sections: &[Section], ring: Ring, tone: impl Fn(usize, Vec3) -> Rgb) {
    let rings: Vec<Vec<Vec3>> = sections.iter().map(|s| ring.corners(s)).collect();
    let heart = |r: &[Vec3]| r.iter().sum::<Vec3>() / r.len() as f32;
    for (span, pair) in rings.windows(2).enumerate() {
        let middle = (heart(&pair[0]) + heart(&pair[1])) * 0.5;
        for i in 0..pair[0].len() {
            let j = (i + 1) % pair[0].len();
            let quad = [pair[0][i], pair[0][j], pair[1][j], pair[1][i]];
            let out = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25 - middle;
            soup.quad(quad, out, tone(span, out.normalize_or_zero()));
        }
    }
    let last = rings.len() - 1;
    lid(soup, &rings[0], Vec3::NEG_Y, tone(0, Vec3::NEG_Y));
    lid(soup, &rings[last], Vec3::Y, tone(last - 1, Vec3::Y));
}
