//! What the plants around the camera shed: the crowns of the canopy within reach of the particle box,
//! nearest first, each with a leaf and a petal material in the colours of its species.

use bevy::prelude::*;

use super::eye::Eye;
use super::kind::Kind;
use super::pool::{ABOVE, BELOW, HALF_WIDTH, Pool};
use super::rate::{self, Air};
use super::rng::Rng;
use crate::world::CHUNK_SIZE;
use crate::world::canopy::{Canopy, Shed};

/// Most plants read at once: one leaf and one petal material each.
pub const SLOTS: usize = 8;

/// Metres past the particle box within which a chunk may hold a crown that reaches into it.
const MARGIN: f32 = 12.0;

/// One plant within reach of the particle box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Source {
    /// Where its stem meets the ground, in the local frame of the eye: `y` is up.
    pub at: Vec3,
    /// Height of the plant in metres.
    pub height: f32,
    /// Radius of its crown in metres.
    pub reach: f32,
    /// What falls from it.
    pub shed: Shed,
    /// Leaves per second it loses right now.
    pub leaf: f32,
    /// Petals per second it loses right now.
    pub petal: f32,
    /// Pollen specks per second it lets go right now.
    pub pollen: f32,
}

/// The plants within reach of the particle box, nearest first, at most [`SLOTS`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shedding {
    /// The plants.
    pub sources: Vec<Source>,
}

impl Shedding {
    /// Sets what each plant sheds per second in `air`.
    pub fn weigh(&mut self, air: &Air) {
        for source in &mut self.sources {
            source.leaf = rate::leaves(&source.shed, air);
            source.petal = rate::petals(&source.shed, air);
            source.pollen = rate::pollen(&source.shed, air);
        }
    }

    /// Returns the leaves per second all the plants within reach lose.
    pub fn leaves(&self) -> f32 {
        self.sources.iter().map(|s| s.leaf).sum()
    }

    /// Returns the petals per second all the plants within reach lose.
    pub fn petals(&self) -> f32 {
        self.sources.iter().map(|s| s.petal).sum()
    }

    /// Returns the pollen specks per second all the plants within reach let go.
    pub fn pollen(&self) -> f32 {
        self.sources.iter().map(|s| s.pollen).sum()
    }

    /// Returns the index of the plant a particle of `kind` comes from, drawn by `roll` in `[0, 1)`
    /// among the plants that shed it, or nothing when none does.
    pub fn choose(&self, kind: Kind, roll: f32) -> Option<usize> {
        let weight = |s: &Source| match kind {
            Kind::Leaf => s.leaf,
            Kind::Petal => s.petal,
            Kind::Pollen => s.pollen,
            _ => 0.0,
        };
        let total: f32 = self.sources.iter().map(weight).sum();
        let mut left = roll * total;
        self.sources.iter().position(|s| {
            left -= weight(s);
            total > 0.0 && left < 0.0
        })
    }
}

/// Returns where a particle of `kind` from `source` starts, in the local frame of the eye: in its crown
/// for a leaf or a petal and low around it for pollen, kept inside the particle box.
pub fn start(source: &Source, kind: Kind, rng: &mut Rng) -> Vec3 {
    let angle = rng.range(0.0, std::f32::consts::TAU);
    let spread = match kind {
        Kind::Pollen => 1.5,
        _ => 1.0,
    };
    let out = source.reach * spread * rng.next().sqrt();
    let rise = match kind {
        Kind::Pollen => rng.range(0.2, 0.6 + source.height),
        _ => source.height * rng.range(0.45, 1.0),
    };
    let at = source.at + Vec3::new(angle.cos() * out, rise, angle.sin() * out);
    let edge = HALF_WIDTH * 0.95;
    Vec3::new(
        at.x.clamp(-edge, edge),
        at.y.clamp(-BELOW, ABOVE),
        at.z.clamp(-edge, edge),
    )
}

/// Returns the plant of `crown` at render position `foot` as the eye sees it, when its crown reaches
/// into the particle box.
fn seen(eye: &Eye, foot: Vec3, crown: &crate::world::canopy::Crown) -> Option<Source> {
    let shed = crown.shed?;
    let at = eye.tilt.inverse() * (foot - eye.at);
    let wide = at.x.abs().max(at.z.abs()) < HALF_WIDTH + crown.reach;
    let tall = at.y + crown.height > -BELOW && at.y < ABOVE;
    (wide && tall).then_some(Source {
        at,
        height: crown.height,
        reach: crown.reach,
        shed,
        leaf: 0.0,
        petal: 0.0,
        pollen: 0.0,
    })
}

/// Sets the base colour of `handle` to the linear RGB `rgb`, only when it changes.
fn tint(
    materials: &mut Assets<StandardMaterial>,
    handle: &Handle<StandardMaterial>,
    rgb: [f32; 3],
) {
    let colour = Color::linear_rgb(rgb[0], rgb[1], rgb[2]);
    if materials
        .get(handle)
        .is_some_and(|m| m.base_color != colour)
        && let Some(mut material) = materials.get_mut(handle)
    {
        material.base_color = colour;
    }
}

/// Reads the plants within reach of the particle box from the canopy of the chunks near the eye, keeps the
/// nearest [`SLOTS`], and paints the leaf and petal material of each slot in the colours of its plant.
pub fn gather(
    eye: Res<Eye>,
    chunks: Query<(&Canopy, &GlobalTransform)>,
    pool: Res<Pool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sensed: ResMut<super::sense::Sensed>,
) {
    let near = CHUNK_SIZE * std::f32::consts::FRAC_1_SQRT_2 + HALF_WIDTH + MARGIN;
    let shed = &mut sensed.shed.sources;
    shed.clear();
    for (canopy, place) in &chunks {
        if place.translation().distance(eye.at) > near {
            continue;
        }
        let crowns = canopy.0.iter();
        shed.extend(crowns.filter_map(|c| seen(&eye, place.transform_point(c.at), c)));
    }
    shed.sort_by(|a, b| {
        a.at.xz()
            .length_squared()
            .total_cmp(&b.at.xz().length_squared())
    });
    shed.truncate(SLOTS);
    for (i, source) in shed.iter().enumerate() {
        if let Some(leaf) = source.shed.leaf {
            tint(&mut materials, &pool.leaves[i], leaf);
        }
        if let Some(flower) = source.shed.flower {
            tint(&mut materials, &pool.petals[i], flower);
        }
    }
}

#[cfg(test)]
#[path = "tests_shed.rs"]
mod tests;
