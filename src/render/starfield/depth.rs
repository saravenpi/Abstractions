//! How far each speck of the sky is, so that the stars slide past a moving camera by their true
//! parallax between two builds of the mesh, and the next build changes nothing on screen.

use std::ops::Deref;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::specks::{Speck, mesh, star_light, star_speck};
use crate::universe::StarSpec;

/// A speck with its depth, as seen from the place the mesh is built for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sighted {
    /// The speck as the mesh draws it.
    pub speck: Speck,
    /// Distance in game light-years.
    pub range: f32,
    /// Light that arrives from a star, as [`star_light`] gives it.
    pub flux: f32,
}

impl Deref for Sighted {
    /// The speck it carries.
    type Target = Speck;

    /// Reads as the speck it carries.
    fn deref(&self) -> &Speck {
        &self.speck
    }
}

impl Sighted {
    /// Returns a star seen from `from`, both in metres from the galaxy centre.
    pub fn star(star: &StarSpec, from: DVec3) -> Self {
        let (range, flux) = star_light(star, from);
        Self {
            speck: star_speck(star, from),
            range: range as f32,
            flux,
        }
    }
}

/// Builds the mesh of the sighted specks: the mesh of their specks, with the range and the
/// light of each in the second pair of texture coordinates of its four corners.
pub fn lit(sighted: &[Sighted]) -> Mesh {
    let specks: Vec<Speck> = sighted.iter().map(|seen| seen.speck).collect();
    let depth = |seen: &Sighted| [[seen.range, seen.flux]; 4];
    let depths: Vec<[f32; 2]> = sighted.iter().flat_map(depth).collect();
    let mut mesh = mesh(&specks);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, depths);
    mesh
}
