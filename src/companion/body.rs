//! The companion's meshes, built in code: a rounded head-body with its screen bezel and antenna, a hand, an orb.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::tune::HALF;

/// Quads along each edge of a face of the rounded body. Four gives 192 triangles.
const GRID: i32 = 4;

/// Half width and half height of the lit screen in meters. The picture is twice as wide as tall.
pub const SCREEN: Vec2 = Vec2::new(0.118, 0.059);

/// Distance of the lit screen in front of the body centre, in meters.
pub const SCREEN_FRONT: f32 = HALF[2] + 0.012;

/// sRGB colours of the shell, its lower part, the screen bezel and the antenna stem.
const PAINT: [[f32; 3]; 4] = [
    [0.94, 0.94, 0.9],
    [0.7, 0.78, 0.86],
    [0.035, 0.04, 0.06],
    [0.45, 0.48, 0.55],
];

/// Flat-shaded triangles with one colour each.
#[derive(Default)]
struct Soup {
    /// Corner positions, three per triangle.
    positions: Vec<[f32; 3]>,
    /// Corner normals, the same for the three corners of a triangle.
    normals: Vec<[f32; 3]>,
    /// Linear corner colours.
    colors: Vec<[f32; 4]>,
}

impl Soup {
    /// Adds a triangle wound so that its front faces `out`.
    fn tri(&mut self, p: [Vec3; 3], out: Vec3, rgb: [f32; 3]) {
        let flip = (p[1] - p[0]).cross(p[2] - p[0]).dot(out) < 0.0;
        let [a, b, c] = if flip { [p[0], p[2], p[1]] } else { p };
        let normal = (b - a).cross(c - a).normalize_or_zero().to_array();
        let color = LinearRgba::from(Color::srgb(rgb[0], rgb[1], rgb[2])).to_f32_array();
        self.positions.extend([a, b, c].map(|p| p.to_array()));
        self.normals.extend([normal; 3]);
        self.colors.extend([color; 3]);
    }

    /// Adds a quad wound so that its front faces `out`.
    fn quad(&mut self, p: [Vec3; 4], out: Vec3, rgb: [f32; 3]) {
        self.tri([p[0], p[1], p[2]], out, rgb);
        self.tri([p[0], p[2], p[3]], out, rgb);
    }

    /// Converts the triangles into a mesh with vertex colours.
    fn into_mesh(self) -> Mesh {
        let count = self.positions.len() as u32;
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32((0..count).collect()))
    }
}

/// Returns the point of the rounded body under the cube point `p`, whose coordinates span -1 to 1.
///
/// The body is a superellipsoid of power four: a box whose edges and corners are fully rounded.
fn round(p: Vec3) -> Vec3 {
    let norm = (p.x.powi(4) + p.y.powi(4) + p.z.powi(4)).powf(0.25);
    p / norm * Vec3::from_array(HALF)
}

/// Adds the six rounded faces of the body, paler above and blue-grey below.
fn shell(soup: &mut Soup) {
    let axes = [Vec3::X, Vec3::Y, Vec3::Z];
    let at = |i: i32| i as f32 / GRID as f32 * 2.0 - 1.0;
    for (face, sign) in (0..3).flat_map(|face| [(face, 1.0), (face, -1.0)]) {
        let (n, u, v) = (
            axes[face] * sign,
            axes[(face + 1) % 3],
            axes[(face + 2) % 3],
        );
        for (i, j) in (0..GRID).flat_map(|i| (0..GRID).map(move |j| (i, j))) {
            let corner = |a: i32, b: i32| round(n + u * at(i + a) + v * at(j + b));
            let p = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
            let middle = (p[0] + p[2]) * 0.5;
            let low = middle.y < -HALF[1] * 0.45;
            soup.quad(p, middle, PAINT[usize::from(low)]);
        }
    }
}

/// Adds the dark bezel the screen sits in: a plate just proud of the front, with a skirt back into the body.
fn bezel(soup: &mut Soup) {
    let (w, h) = (SCREEN.x + 0.016, SCREEN.y + 0.016);
    let (front, back) = (-(SCREEN_FRONT - 0.002), -HALF[2] * 0.7);
    let ring = |z: f32, grow: f32| {
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(x, y): (f32, f32)| Vec3::new(x * (w + grow), y * (h + grow), z))
    };
    let (near, far) = (ring(front, 0.0), ring(back, 0.012));
    soup.quad(near, Vec3::NEG_Z, PAINT[2]);
    for side in 0..4 {
        let next = (side + 1) % 4;
        let p = [near[side], near[next], far[next], far[side]];
        soup.quad(
            p,
            (near[side] + near[next]) * Vec3::new(1.0, 1.0, 0.0),
            PAINT[2],
        );
    }
}

/// Adds the antenna stem: a thin three-sided rod from the top of the head to `tip`.
fn antenna(soup: &mut Soup, tip: Vec3) {
    let foot = Vec3::Y * HALF[1] * 0.96;
    let rim = |centre: Vec3, i: usize| {
        let a = i as f32 / 3.0 * TAU;
        centre + Vec3::new(a.cos(), 0.0, a.sin()) * 0.009
    };
    for side in 0..3 {
        let p = [
            rim(foot, side),
            rim(foot, side + 1),
            rim(tip, side + 1),
            rim(tip, side),
        ];
        let out = (p[0] + p[1]) * 0.5 - foot;
        soup.quad(p, out, PAINT[3]);
    }
}

/// Builds the head-body with its screen bezel and antenna stem. `tip` is where the antenna ends.
pub fn body(tip: Vec3) -> Mesh {
    let mut soup = Soup::default();
    shell(&mut soup);
    bezel(&mut soup);
    antenna(&mut soup, tip);
    soup.into_mesh()
}
