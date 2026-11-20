//! The mesh of the signs: one flat mesh of [`QUADS`] quads, made once and written in place at every
//! lay, so drawing the signs never allocates once it is made.

use bevy::asset::RenderAssetUsages;
use bevy::math::{DVec3, Vec2, Vec3};
use bevy::mesh::VertexAttributeValues::{Float32x3, Float32x4};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::shapes::{Quad, quads};
use super::tune::{LIFT, QUADS};
use crate::fauna::place::Spot;
use crate::life::signs::Sign;

/// Corners of every quad, in the order of its two triangles' indices.
const CORNERS: [(f32, f32); 4] = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];

/// Where signs are painted: the flat frame of the mesh, the radius of the planet, and the height in
/// the frame of the drawn ground over `(x, z)`.
pub struct Canvas<'a> {
    /// The frame of the mesh.
    pub spot: Spot,
    /// Sea level radius of the planet in metres.
    pub radius: f64,
    /// Height in the frame of the drawn ground over `(x, z)`.
    pub ground: &'a dyn Fn(Vec2) -> f32,
    /// Linear RGB colour of the blood of a species, at a freshness from 1 (just fell) to 0 (dried).
    pub blood: &'a dyn Fn(u16, f32) -> [f32; 3],
}

/// Builds the empty mesh of the signs: [`QUADS`] quads all folded onto the origin.
pub fn mesh() -> Mesh {
    let corners = QUADS * 4;
    let indices = (0..QUADS as u32).flat_map(|q| [0, 1, 2, 0, 2, 3].map(|i| q * 4 + i));
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; corners])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; corners])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32; 4]; corners])
    .with_inserted_indices(Indices::U32(indices.collect()))
}

/// Returns the material of the signs: their colours and opacity are in the mesh.
pub fn material() -> StandardMaterial {
    StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        cull_mode: None,
        ..default()
    }
}

/// Writes `signs` into `mesh` on `canvas`, nearest first, and returns how many quads it drew: at
/// most [`QUADS`], the rest folded away. The buffers of the mesh are taken out and put back, so their
/// room is kept.
pub fn paint(mesh: &mut Mesh, signs: &[Sign], canvas: &Canvas) -> usize {
    let (Some(Float32x3(mut positions)), Some(Float32x4(mut colours))) = (
        mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION),
        mesh.remove_attribute(Mesh::ATTRIBUTE_COLOR),
    ) else {
        return 0;
    };
    positions.clear();
    colours.clear();
    for sign in signs {
        let lie = flat(canvas, sign.at, sign.facing);
        let blood = (canvas.blood)(sign.species, sign.fresh);
        quads(sign, lie, blood, &mut |q| {
            put((&mut positions, &mut colours), &q, canvas)
        });
    }
    let drawn = positions.len() / 4;
    positions.resize(QUADS * 4, [0.0; 3]);
    colours.resize(QUADS * 4, [0.0; 4]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    drawn
}

/// Returns unit direction `at` and the unit tangent `facing` there as a point and a unit direction
/// `(x, z)` in the frame of `canvas`.
fn flat(canvas: &Canvas, at: DVec3, facing: DVec3) -> (Vec2, Vec2) {
    let local = canvas.spot.local(at * canvas.radius);
    let way = (canvas.spot.rot.inverse() * facing).as_vec3();
    (local.xz(), way.xz().normalize_or(Vec2::NEG_Y))
}

/// Adds the four corners of `quad` on the ground of `canvas` and their colour to the buffers, unless
/// they are full.
fn put(
    (positions, colours): (&mut Vec<[f32; 3]>, &mut Vec<[f32; 4]>),
    quad: &Quad,
    canvas: &Canvas,
) {
    if positions.len() >= QUADS * 4 {
        return;
    }
    colours.extend([quad.colour; 4]);
    let across = quad.along.perp();
    for (u, v) in CORNERS {
        let xz = quad.centre + quad.along * (u * quad.half.x) + across * (v * quad.half.y);
        let y = (canvas.ground)(xz) + LIFT;
        positions.push(Vec3::new(xz.x, y, xz.y).to_array());
    }
}
