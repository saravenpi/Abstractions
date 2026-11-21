//! The plant cover of a chunk of a living planet: where its plants stand thick, where they thin out to lone
//! specimens and where the ground lies barren, which species of each group holds a patch, and how coasts,
//! slopes and the shade of big plants change that.
//!
//! The cover follows a slow noise of barren stretches and a faster one of clearings, lifted where the air is
//! wet, and cut at a level the patchiness of the planet sets: a lush planet is mostly covered, a patchy one
//! holds its life in scattered stands. It is read at the corners of a coarse grid per chunk and eased between.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use super::air::Air;
use super::pass::Ground;
use crate::climate::Climate;
use crate::life::Tolerance;
use crate::world::{CHUNK_SIZE, Terrain};

/// Points along one side of the chunk at which the cover is read.
const POINTS: usize = 5;

/// Frequencies per metre of the noise of barren stretches, of clearings and of the stands of each species.
const FREQ: [f64; 3] = [1.0 / 380.0, 1.0 / 70.0, 1.0 / 45.0];

/// Salts of the planet seed the three noises are drawn from.
const SALTS: [u32; 3] = [0xC0E5, 0x61AD, 0x57A0];

/// Cover level a lush planet is cut at, and how much higher the patchiest planet cuts it.
const CUT: (f32, f32) = (0.12, 0.6);

/// Half the width of the easing between barren and covered ground, in cover level.
const SOFT: f32 = 0.1;

/// Share of its covered chance a species keeps on barren ground: lone specimens.
const LONE: f32 = 0.05;

/// Chance of a species on covered ground against before: on dry ground, and more per unit of moisture.
const LUSH: (f32, f32) = (1.45, 0.6);

/// Metres above the sea within which a coast is lusher.
const COAST: f32 = 3.0;

/// Metres from a big plant, past its room, within which the undergrowth is thicker.
const SHADE: f32 = 2.5;

/// Kelvin outside its tolerance at which a group no longer grows.
const WARMTH: f32 = 8.0;

/// Moisture outside its tolerance at which a group no longer grows.
const WETNESS: f32 = 0.15;

/// The cover over one chunk: at each grid point, how covered the ground is (0 to 1), the kin key that picks
/// the species of a stand (0 to 1), and the moisture of the air.
pub struct Cover {
    /// Cover, kin key and moisture at each point, the first index along x and the second along z.
    grid: [[[f32; 3]; POINTS]; POINTS],
    /// Chart position of the low corner of the chunk.
    corner: Vec2,
}

/// Returns the share `x` has moved from `a` to `b`, eased and clamped to 0 to 1.
fn ease(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The noises the cover is read from on one planet, so that single places can be read without a chunk.
pub struct Sampler {
    /// The noises of barren stretches, of clearings and of the stands of each species.
    fields: [Perlin; 3],
    /// Cover level the planet is cut at.
    cut: f32,
}

impl Sampler {
    /// Returns the sampler of `terrain` on a planet of `patchiness`.
    pub fn new(terrain: &Terrain, patchiness: f32) -> Self {
        Self {
            fields: SALTS.map(|salt| Perlin::new(terrain.seed() as u32 ^ salt)),
            cut: CUT.0 + CUT.1 * patchiness,
        }
    }

    /// Returns the cover, kin key and moisture `wet` at chart position `w` of `terrain`.
    pub fn at(&self, terrain: &Terrain, w: Vec2, wet: f32) -> [f32; 3] {
        let p = terrain.embed(w.x, 0.0, w.y);
        let read = |f: usize| self.fields[f].get((p * FREQ[f]).to_array()) as f32;
        let level = 0.5 + 0.8 * read(0) + 0.35 * read(1) + 0.3 * (wet - 0.5);
        let key = (0.5 + 1.3 * read(2)).clamp(0.0, 1.0);
        [ease(self.cut - SOFT, self.cut + SOFT, level), key, wet]
    }
}

impl Cover {
    /// Reads the cover of chunk `coord` of `terrain` on a planet of `patchiness`, in the climate `air`.
    pub fn new(terrain: &Terrain, coord: IVec2, patchiness: f32, air: Option<&Air>) -> Self {
        let sampler = Sampler::new(terrain, patchiness);
        let corner = coord.as_vec2() * CHUNK_SIZE;
        let step = CHUNK_SIZE / (POINTS - 1) as f32;
        let at = |i: usize, j: usize| {
            let w = corner + Vec2::new(i as f32, j as f32) * step;
            let wet = air.map_or(0.5, |a| a.at(w).moisture.min(1.2));
            sampler.at(terrain, w, wet)
        };
        Self {
            grid: std::array::from_fn(|i| std::array::from_fn(|j| at(i, j))),
            corner,
        }
    }

    /// Returns the cover, kin key and moisture at chart position `w` of the chunk, eased between the points.
    pub fn at(&self, w: Vec2) -> [f32; 3] {
        let top = (POINTS - 1) as f32 - 1e-3;
        let t = ((w - self.corner) / (CHUNK_SIZE / (POINTS - 1) as f32))
            .clamp(Vec2::ZERO, Vec2::splat(top));
        let (i, j, f) = (t.x as usize, t.y as usize, t.fract());
        let g = &self.grid;
        std::array::from_fn(|k| {
            let low = g[i][j][k] + (g[i + 1][j][k] - g[i][j][k]) * f.x;
            let high = g[i][j + 1][k] + (g[i + 1][j + 1][k] - g[i][j + 1][k]) * f.x;
            low + (high - low) * f.y
        })
    }
}

/// Returns how thick a species grows where the cover is `cover` and the moisture `wet`: lone specimens on
/// barren ground, thicker than before under full cover, and thickest in wet lowlands.
pub fn thickness(cover: f32, wet: f32) -> f32 {
    LONE + (LUSH.0 + LUSH.1 * wet.min(1.0) - LONE) * cover
}

/// Returns how a plant of the big pass when `big` does on `at`, `rise` metres above the sea: lusher along a
/// coast, and big plants thinner on steep slopes.
pub fn edge(at: &Ground, rise: f32, big: bool) -> f32 {
    let coast = (0.0..COAST).contains(&rise);
    let shore = match (coast, big) {
        (true, true) => 1.15,
        (true, false) => 1.4,
        (false, _) => 1.0,
    };
    let steep = if big {
        (1.0 - 0.7 * at.slope).max(0.25)
    } else {
        1.0
    };
    shore * steep
}

/// Returns how thick the undergrowth is at chart position `w` among the big plants `canopy`, each with its
/// position and room: thicker round them, thinner in the open.
pub fn shade(canopy: &[(Vec2, f32)], w: Vec2) -> f32 {
    match canopy.iter().any(|(c, room)| c.distance(w) < room + SHADE) {
        true => 1.8,
        false => 0.85,
    }
}

/// Returns how well tolerance `t` stands `air`, from 1 inside it to 0 a few kelvin or a little moisture
/// outside. Without a climate every group stands it.
pub fn fit(t: &Tolerance, air: Option<&Climate>) -> f32 {
    let Some(air) = air else {
        return 1.0;
    };
    let heat = (t.cold - air.temperature)
        .max(air.temperature - t.heat)
        .max(0.0);
    let damp = (t.dry - air.moisture).max(air.moisture - t.wet).max(0.0);
    (1.0 - heat / WARMTH).max(0.0) * (1.0 - damp / WETNESS).max(0.0)
}
