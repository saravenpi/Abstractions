//! The loft: a shape through cross-sections along the Y axis, its rings rounded rectangles.

use std::f32::consts::TAU;

use bevy::prelude::*;

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
    pub fn new(y: f32, x: f32, z: f32) -> Self {
        Self {
            y,
            at: Vec2::ZERO,
            half: Vec2::new(x, z),
        }
    }

    /// Returns the section moved by `x` and `z`.
    pub fn shifted(mut self, x: f32, z: f32) -> Self {
        self.at += Vec2::new(x, z);
        self
    }

    /// Returns corner `i` of `sides` of the ring, a superellipse of `power`: 2 is round, more is squarer.
    fn corner(&self, i: u32, sides: u32, power: f32) -> Vec3 {
        let angle = (i % sides) as f32 / sides as f32 * TAU;
        let curve = |t: f32| t.signum() * t.abs().powf(2.0 / power);
        let (x, z) = (curve(angle.cos()), curve(angle.sin()));
        Vec3::new(self.at.x + x * self.half.x, self.y, self.at.y + z * self.half.y)
    }
}

/// Adds a lofted shape through `sections`, ordered along Y, of `sides` sides and ring `power`. The
/// side `i` of the span `s` takes `tone(s, i)`. With `capped`, both ends are closed.
pub fn lathe(
    soup: &mut Soup,
    sections: &[Section],
    (sides, power, capped): (u32, f32, bool),
    tone: impl Fn(usize, u32) -> [f32; 3],
) {
    let ring = |s: &Section| -> Vec<Vec3> { (0..sides).map(|i| s.corner(i, sides, power)).collect() };
    let rings: Vec<Vec<Vec3>> = sections.iter().map(ring).collect();
    let heart = |r: &[Vec3]| r.iter().sum::<Vec3>() / r.len() as f32;
    for (span, pair) in rings.windows(2).enumerate() {
        let middle = (heart(&pair[0]) + heart(&pair[1])) * 0.5;
        for i in 0..sides as usize {
            let j = (i + 1) % sides as usize;
            let quad = [pair[0][i], pair[0][j], pair[1][j], pair[1][i]];
            let out = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25 - middle;
            soup.quad(quad, out, tone(span, i as u32));
        }
    }
    let ends = [(0, 1), (rings.len() - 1, rings.len().saturating_sub(2))];
    for (end, (at, next)) in ends.into_iter().enumerate().filter(|_| capped) {
        let (c, out) = (heart(&rings[at]), heart(&rings[at]) - heart(&rings[next]));
        let span = if end == 0 { 0 } else { rings.len() - 2 };
        for i in 0..sides as usize {
            let j = (i + 1) % sides as usize;
            soup.tri([c, rings[at][i], rings[at][j]], out, tone(span, i as u32));
        }
    }
}

/// Adds a drum of `radius` and half length `half` around `at`, its axis along `axis`, of `sides`
/// sides: a joint housing. The caps are closed.
pub fn drum(soup: &mut Soup, (at, axis): (Vec3, Vec3), (radius, half): (f32, f32), (sides, rgb): (u32, [f32; 3])) {
    let mut body = Soup::default();
    let rings = [Section::new(-half, radius, radius), Section::new(half, radius, radius)];
    lathe(&mut body, &rings, (sides, 2.0, true), |_, _| rgb);
    soup.extend(&body, (at, Quat::from_rotation_arc(Vec3::Y, axis.normalize())));
}

/// Adds a ball of `radius` around `at`, of `sides` sides: five rings, a joint cap that reads round.
pub fn orb(soup: &mut Soup, at: Vec3, radius: f32, (sides, rgb): (u32, [f32; 3])) {
    let ring = |y: f32, r: f32| Section::new(y * radius, r * radius, r * radius);
    let rings = [ring(-0.95, 0.3), ring(-0.6, 0.8), ring(0.0, 1.0), ring(0.6, 0.8), ring(0.95, 0.3)];
    let mut body = Soup::default();
    lathe(&mut body, &rings, (sides, 2.0, true), |_, _| rgb);
    soup.extend(&body, (at, Quat::IDENTITY));
}
