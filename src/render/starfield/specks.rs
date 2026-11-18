//! Specks of light on the sky: the nearby stars and the far galaxies, and the one mesh that draws them.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::universe::{GALAXY_CELL, GLY, GalaxyKind, GalaxySpec, StarSpec};

/// Largest angular radius in radians a galaxy is drawn with.
pub const GALAXY_ANGLE: f32 = 0.35;

/// One point of light on the sky shell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speck {
    /// Unit vector toward it, in galaxy axes.
    pub dir: Vec3,
    /// Angular radius in radians.
    pub size: f32,
    /// Linear colour times brightness.
    pub color: Vec3,
    /// How fast it fades from its centre: low for a crisp star, high for a soft galaxy.
    pub soft: f32,
}

/// Returns the distance in game light-years of a star seen from `from`, both in metres from the
/// galaxy centre, and the light that arrives from it: the base 10 logarithm of its light output
/// over the squared distance.
pub fn star_light(star: &StarSpec, from: DVec3) -> (f64, f32) {
    let distance = ((star.pos - from).length() / GLY).max(1e-3);
    let flux = (star.luminosity / (distance * distance)).max(1e-9).log10() as f32;
    (distance, flux)
}

/// Returns the angular radius in radians and the strength of the colour a star is drawn with
/// when the light `flux` arrives from it, on a log scale, like magnitudes. `starfield.wesl`
/// repeats this rule, so a star keeps the right size while the camera closes on it.
pub fn shine(flux: f32) -> (f32, f32) {
    let bright = ((flux + 4.6) / 5.0).clamp(0.08, 1.0);
    (0.0035 + 0.0055 * bright, 0.45 + 2.6 * bright * bright)
}

/// Returns the speck of a star seen from `from`, both in metres from the galaxy centre.
///
/// The light that arrives sets both the size and the strength, and the class gives the colour.
pub fn star_speck(star: &StarSpec, from: DVec3) -> Speck {
    let (size, gain) = shine(star_light(star, from).1);
    Speck {
        dir: (star.pos - from).normalize_or(DVec3::Y).as_vec3(),
        size,
        color: star.color * gain,
        soft: 0.6,
    }
}

/// Returns the speck of a galaxy whose centre lies `offset` metres from the camera, or nothing
/// when the camera is inside it.
pub fn galaxy_speck(galaxy: &GalaxySpec, offset: DVec3) -> Option<Speck> {
    let distance = offset.length();
    if distance <= galaxy.radius {
        return None;
    }
    let color = match galaxy.kind {
        GalaxyKind::Spiral => Vec3::new(0.7, 0.8, 1.0),
        GalaxyKind::Elliptical => Vec3::new(1.0, 0.9, 0.7),
        GalaxyKind::Irregular => Vec3::new(0.8, 0.7, 1.0),
    };
    let bright = (0.5 * GALAXY_CELL / distance).clamp(0.1, 0.7) as f32;
    Some(Speck {
        dir: (offset / distance).as_vec3(),
        size: ((galaxy.radius / distance) as f32).min(GALAXY_ANGLE),
        color: color * bright,
        soft: 2.5,
    })
}

/// Builds the mesh of the specks on a unit shell: one quad each, facing the centre, with the
/// colour in the vertex colours and the softness in their alpha.
pub fn mesh(specks: &[Speck]) -> Mesh {
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let mut positions = Vec::with_capacity(specks.len() * 4);
    let mut normals = Vec::with_capacity(specks.len() * 4);
    let mut uvs = Vec::with_capacity(specks.len() * 4);
    let mut colors = Vec::with_capacity(specks.len() * 4);
    let mut indices = Vec::with_capacity(specks.len() * 6);
    for (i, speck) in specks.iter().enumerate() {
        let side = speck.dir.any_orthonormal_vector();
        let rise = speck.dir.cross(side);
        for (x, y) in corners {
            positions.push((speck.dir + (side * x + rise * y) * speck.size).to_array());
            normals.push((-speck.dir).to_array());
            uvs.push([0.5 + 0.5 * x, 0.5 + 0.5 * y]);
            colors.push(speck.color.extend(speck.soft).to_array());
        }
        let first = (i * 4) as u32;
        indices.extend([first, first + 2, first + 1, first, first + 3, first + 2]);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh.enable_raytracing = false;
    mesh
}
