//! The canopy of a chunk: where its big props stand and how much ground they take, how far the crowns of its
//! grown plants reach and what falls from them, read from the looks of their species, so that only real
//! leaves and real flowers come down.
//!
//! The legacy plants of home, rocks and wrecks shed nothing: they have no species.

use bevy::prelude::*;

use super::Terrain;
use super::propmesh::grown::{Look, habit, look};
use super::props::footprint;
use super::scatter::{Placement, Rig, chunk_center};
use crate::life::flora::{self, KELP, Stage};

/// The crystal lattice architecture: shards, not leaves.
const LATTICE: u8 = 7;

/// Most small flowering plants one chunk lists: enough to tell where a meadow is in flower.
const SMALL_MOST: usize = 96;

/// Edge in metres of a fallen needle.
const NEEDLE: f32 = 0.04;

/// Edge in metres of a leaf of leaf size 0, and how much it grows per unit of leaf size.
const LEAF_EDGE: (f32, f32) = (0.05, 0.15);

/// Share of the height of a plant its crown reaches out from the stem, at a spread of 1.
const CROWN: f32 = 0.35;

/// Narrowest crown radius in metres.
const NARROW: f32 = 0.6;

/// How many more leaves fall from a deciduous plant in autumn than in another season.
const AUTUMN: f32 = 3.0;

/// What falls from one plant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shed {
    /// Linear RGB of its leaves, or none for a bare plant or one whose leaves lie on the ground.
    pub leaf: Option<[f32; 3]>,
    /// Edge length in metres of one fallen leaf.
    pub size: f32,
    /// Linear RGB of its flowers, or none out of flower.
    pub flower: Option<[f32; 3]>,
    /// How many leaves fall from it against an evergreen of its size, 1 or more.
    pub rate: f32,
}

/// One prop of the canopy, in the frame of its chunk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crown {
    /// Where its stem meets the ground, in the chunk frame.
    pub at: Vec3,
    /// Height of the plant in metres.
    pub height: f32,
    /// Radius of its crown in metres.
    pub reach: f32,
    /// Radius in metres of the ground its trunk, stem or body takes.
    pub foot: f32,
    /// What falls from it, if anything.
    pub shed: Option<Shed>,
}

/// The big props of one chunk and the small plants in flower, in the frame of the chunk.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Canopy(pub Vec<Crown>);

/// Returns the stage of the year the plants of `terrain` stand at on chart position `center`.
pub fn stage_at(terrain: &Terrain, center: Vec2) -> Stage {
    let globe = terrain.globe();
    let latitude = terrain
        .chart()
        .dir(center.x, center.y)
        .y
        .clamp(-1.0, 1.0)
        .asin() as f32;
    flora::stage(globe.season(), latitude, globe.record().tilt as f32)
}

/// Returns what falls from a plant of `look` at `stage`, standing tall when `tall`: leaves only from a
/// leafy plant that holds them above the ground, flowers from any plant in flower.
pub fn shed(look: &Look, stage: Stage, tall: bool) -> Option<Shed> {
    let dials = &look.dials;
    if matches!(dials.architecture, KELP | LATTICE) {
        return None;
    }
    let form = &dials.form;
    let leaf = dials.paint.leaves.filter(|_| tall);
    let flower = dials.paint.flowers;
    let autumn = form.deciduous && stage == Stage::Fruit;
    let size = match form.needles {
        true => NEEDLE,
        false => LEAF_EDGE.0 + LEAF_EDGE.1 * form.leaf,
    };
    (leaf.is_some() || flower.is_some()).then_some(Shed {
        leaf,
        size,
        flower,
        rate: if autumn { AUTUMN } else { 1.0 },
    })
}

/// Returns the crown of placement `p` placed through `rig` at `stage`, standing tall when `tall`.
fn crown(p: &Placement, rig: &Rig, stage: Stage, tall: bool) -> Crown {
    let look = look(p.variant);
    let height = look.map_or(p.scale, |l| habit(l.dials.architecture).reference * p.scale);
    let spread = look.map_or(1.0, |l| l.dials.kind.spread);
    Crown {
        at: rig.frame(p.pos).0,
        height,
        reach: (CROWN * height * spread).max(NARROW),
        foot: if tall { footprint(p) } else { 0.0 },
        shed: look.and_then(|l| shed(&l, stage, tall)),
    }
}

/// Returns the canopy of chunk `coord` of `terrain` from its big `props` and its small `grass`, placed
/// through `rig`: every big prop, and the first small plants in flower.
pub fn canopy_on(
    terrain: &Terrain,
    coord: IVec2,
    (props, grass): (&[Placement], &[Placement]),
    rig: &Rig,
) -> Canopy {
    let stage = stage_at(terrain, chunk_center(coord));
    let big = props.iter().map(|p| crown(p, rig, stage, true));
    let small = grass
        .iter()
        .map(|p| crown(p, rig, stage, false))
        .filter(|c| c.shed.is_some())
        .take(SMALL_MOST);
    Canopy(big.chain(small).collect())
}

#[cfg(test)]
#[path = "tests_canopy.rs"]
mod tests;
