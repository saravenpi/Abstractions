//! The turf of a living planet: the look of the carpet its ground-cover species lay on open ground, and
//! how thick that carpet grows on each part of a chunk.
//!
//! Ground cover is the mats and the rosettes of the planet that live out of the liquid. A planet without
//! them, a dead planet and the legacy plane have no turf.

use bevy::prelude::*;

use super::air::{Air, still};
use super::chunk_center;
use super::cover::Cover;
use super::grow::{Flora, Grower};
use crate::life::Habitat;
use crate::life::flora::MAT;
use crate::world::Terrain;
use crate::world::propmesh::grown::{Look, look};

/// The rosette architecture: tufts of blades from one root.
const ROSETTE: u8 = 1;

/// Total chance per try of the ground cover of the lushest planet, whose carpet is as thick as it gets.
const FULL: f32 = 0.5;

/// Tallest blade in metres a rosette carpet grows: its taller leaves stand as plants of their own.
const TUFT: f32 = 0.6;

/// Share a second colour is darkened by when the planet has only one ground-cover species.
const SHADOW: f32 = 0.7;

/// The look of the carpet of a planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turf {
    /// Linear RGB of the commonest ground-cover species and of the next one.
    pub colours: [[f32; 3]; 2],
    /// Height in metres of a grown blade.
    pub height: f32,
    /// How thick the carpet grows on fully covered ground, 0 to 1.
    pub density: f32,
    /// How stiffly its blades stand against the wind, 0 (limp) to 1 (rigid).
    pub stiffness: f32,
}

/// Returns true when `g` carpets the ground: a mat or a rosette out of the liquid.
fn carpets(g: &Grower) -> bool {
    (g.arch == MAT || g.arch == ROSETTE) && g.habitat != Habitat::Water
}

/// Returns the height in metres of a blade of a ground-cover species of `look`: a mat as tall as it grows,
/// a rosette no taller than [`TUFT`].
fn blade(look: &Look) -> f32 {
    let height = look.dials.form.height;
    match look.dials.architecture {
        ROSETTE => height.min(TUFT),
        _ => height,
    }
}

/// Returns the turf the ground-cover species of `flora` lay, or nothing when it has none: the colours of
/// the two commonest, blades as tall as the tallest, and as thick as their chances to grow.
pub fn turf_of(flora: &Flora) -> Option<Turf> {
    let mut cover: Vec<&Grower> = flora.growers.iter().filter(|g| carpets(g)).collect();
    cover.sort_by(|a, b| (b.chance * b.share).total_cmp(&(a.chance * a.share)));
    let looks: Vec<_> = cover.iter().filter_map(|g| look(g.looks[1])).collect();
    let first = looks.first()?;
    let paint = |l: &Look| l.dials.paint.leaves.unwrap_or(l.dials.paint.stem);
    let colour = paint(first);
    let second = looks.get(1).map_or(colour.map(|c| c * SHADOW), paint);
    let form = &first.dials.form;
    let tall = looks.iter().map(blade).fold(0.0, f32::max);
    let thick: f32 = cover.iter().map(|g| g.chance * g.share).sum();
    Some(Turf {
        colours: [colour, second],
        height: tall,
        density: (thick / FULL).clamp(0.0, 1.0),
        stiffness: (0.35
            + 0.35 * f32::from(u8::from(form.wax))
            + 0.3 * f32::from(u8::from(form.spines)))
        .min(1.0),
    })
}

/// The turf of one chunk: its look and how thick it grows across the chunk.
pub struct Carpet {
    /// The look of the turf of the planet at the season of the chunk.
    pub turf: Turf,
    /// The plant cover over the chunk.
    cover: Cover,
}

impl Carpet {
    /// Returns the turf of chunk `coord` of `terrain`, or nothing where no ground cover grows.
    pub fn of(terrain: &Terrain, coord: IVec2) -> Option<Self> {
        let flora = Flora::of(terrain, chunk_center(coord))?;
        let turf = turf_of(&flora)?;
        let air = (!still()).then(|| Air::new(terrain, coord));
        Some(Self {
            turf,
            cover: Cover::new(terrain, coord, flora.patchiness, air.as_ref()),
        })
    }

    /// Returns how thick the turf grows at chart position `w` of the chunk, 0 (barren) to 1.
    pub fn at(&self, w: Vec2) -> f32 {
        self.cover.at(w)[0] * self.turf.density.sqrt()
    }
}
