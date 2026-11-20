//! The three shapes the body is made of: a rounded block, a tube and a small ball.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::skin::Soup;

/// A tube between two points, wider or narrower at each end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tube {
    /// Centres of the two ends.
    pub ends: (Vec3, Vec3),
    /// Radius at each end.
    pub radii: (f32, f32),
    /// Number of sides.
    pub sides: u32,
    /// Whether the ends are closed. An end buried in a joint needs no lid.
    pub capped: bool,
}

impl Tube {
    /// Returns an open six-sided tube from `a` to `b` with the radii `radii`.
    pub fn open(a: [f32; 3], b: [f32; 3], radii: (f32, f32)) -> Self {
        let ends = (Vec3::from_array(a), Vec3::from_array(b));
        Self {
            ends,
            radii,
            sides: 6,
            capped: false,
        }
    }

    /// Returns a closed drum of `sides` sides from `a` to `b` with the radius `radius`.
    pub fn drum(a: [f32; 3], b: [f32; 3], radius: f32, sides: u32) -> Self {
        let ends = (Vec3::from_array(a), Vec3::from_array(b));
        Self {
            ends,
            radii: (radius, radius),
            sides,
            capped: true,
        }
    }

    /// Returns corner `i` of the ring at the first end (`far` false) or the second.
    fn corner(&self, i: u32, far: bool) -> Vec3 {
        let axis = (self.ends.1 - self.ends.0).normalize_or_zero();
        let across = axis.any_orthonormal_vector();
        let angle = i as f32 / self.sides as f32 * TAU;
        let spoke = across * angle.cos() + axis.cross(across) * angle.sin();
        match far {
            false => self.ends.0 + spoke * self.radii.0,
            true => self.ends.1 + spoke * self.radii.1,
        }
    }
}

/// Adds a tube of the sRGB colour `rgb`: two triangles per side, and one more per side and lid when closed.
pub fn tube(soup: &mut Soup, tube: &Tube, rgb: [f32; 3]) {
    let (a, b) = tube.ends;
    for i in 0..tube.sides {
        let near = [tube.corner(i, false), tube.corner(i + 1, false)];
        let far = [tube.corner(i, true), tube.corner(i + 1, true)];
        let out = (near[0] + near[1] + far[0] + far[1]) * 0.25 - (a + b) * 0.5;
        soup.quad(
            [near[0], near[1], far[1], far[0]],
            out.reject_from(b - a),
            rgb,
        );
        if tube.capped {
            soup.tri([a, near[0], near[1]], a - b, rgb);
            soup.tri([b, far[0], far[1]], b - a, rgb);
        }
    }
}

/// Adds a rounded block around `centre` with the half extents `half`: a box whose edges and
/// corners are fully rounded, `grid` quads along each edge, coloured by `paint` from a point.
pub fn rounded(
    soup: &mut Soup,
    (centre, half): (Vec3, Vec3),
    grid: i32,
    paint: impl Fn(Vec3) -> [f32; 3],
) {
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];
    let at = |i: i32| i as f32 / grid as f32 * 2.0 - 1.0;
    let round = |p: Vec3| p / (p.x.powi(4) + p.y.powi(4) + p.z.powi(4)).powf(0.25) * half;
    for (face, sign) in (0..3).flat_map(|face| [(face, 1.0), (face, -1.0)]) {
        let (n, u, v) = (
            axes[face] * sign,
            axes[(face + 1) % 3],
            axes[(face + 2) % 3],
        );
        for (i, j) in (0..grid).flat_map(|i| (0..grid).map(move |j| (i, j))) {
            let corner = |a: i32, b: i32| round(n + u * at(i + a) + v * at(j + b));
            let p = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
            let middle = (p[0] + p[2]) * 0.5;
            soup.quad(p.map(|p| p + centre), middle, paint(middle / half));
        }
    }
}

/// Adds a ball of `radius` around `centre`: an octahedron, enough for a joint cap.
pub fn ball(soup: &mut Soup, centre: [f32; 3], radius: f32, rgb: [f32; 3]) {
    let centre = Vec3::from_array(centre);
    for (x, y, z) in (0..8).map(|i| (i & 1, (i >> 1) & 1, i >> 2)) {
        let sign = |bit: i32| 1.0 - 2.0 * bit as f32;
        let corners = [Vec3::X * sign(x), Vec3::Y * sign(y), Vec3::Z * sign(z)];
        let out = corners[0] + corners[1] + corners[2];
        soup.tri(corners.map(|p| centre + p * radius), out, rgb);
    }
}
