//! The large shape of the ground: the plates, the shelf between a continent and its sea floor, the regional
//! band and the shore, summed into one height in metres by direction.
//!
//! The shore is measured, not tabled: the crust blend of the plates is offset until the sea covers the
//! share of the planet its record gives it.
//!
//! The local ground still draws every coast. Within [`tune::FRINGE`] of the sea the large height only
//! pushes the coast measure of the local ground towards land or sea; beyond it the rest of the large height
//! is added to the local ground, which then is all land or all sea floor.

mod band;
pub mod tune;

use std::sync::OnceLock;

use bevy::math::DVec3;

use super::datum::{shore, sites};
use super::plates::{Interior, Plates};
use super::relief::Scales;
use crate::world::islands::smooth;
use band::Band;
use tune::{DRY, FRINGE, INLAND, PLAIN, SHARPEN, SHELF};

/// What the large shape of the ground does to the local ground at one place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lift {
    /// Added to the coast measure of the local ground: -1 far out at sea, 1 well inland.
    pub bias: f32,
    /// Metres added to the height of the local ground.
    pub height: f32,
}

/// Returns the inland measure the climate reads for the coast measure `base` of the local ground: `base`
/// itself on a world the local ground shapes alone, and kept within the span the climate was tuned on
/// where a `lift` has pushed it.
pub fn inland(lift: Option<Lift>, base: f32) -> f32 {
    match lift {
        Some(_) => base.clamp(INLAND.0, INLAND.1),
        None => base,
    }
}

/// The large shape of the ground of one planet.
pub struct Elevation {
    /// The plates of the planet.
    plates: Plates,
    /// Sea level radius in metres.
    radius: f64,
    /// The heights the size and the pull of the planet allow.
    scales: Scales,
    /// The regional noise.
    band: Band,
    /// Offset on the crust blend that puts the low share of the planet under the sea, measured when first
    /// read.
    shore: OnceLock<f32>,
}

impl Elevation {
    /// Returns the large shape of planet `seed` of `radius` metres, with `scales` and `interior`.
    pub fn new(seed: u64, radius: f64, scales: Scales, interior: Interior) -> Self {
        Self {
            plates: Plates::new(seed, radius, interior),
            radius,
            scales,
            band: Band::new(seed, radius),
            shore: OnceLock::new(),
        }
    }

    /// Returns the plates of the planet.
    pub fn plates(&self) -> &Plates {
        &self.plates
    }

    /// Returns the two parts of the large height in unit direction `dir`: the crust blend moved by the
    /// coast waves, which the shore offsets, and the metres the shore leaves alone, which are the ranges and
    /// volcanoes broken into crests, the rifts, the sea floor shapes, the basins and the regional band.
    fn parts(&self, dir: DVec3) -> (f32, f32) {
        let s = self.plates.sample(dir);
        let (p, k) = (dir * self.radius, &self.scales);
        let high = s.rise.max(0.0) + s.cone;
        let crests = match high > 0.0 {
            true => self.band.ridge(p),
            false => 1.0,
        };
        let rest = (high * crests + s.rise.min(0.0)) * k.peak
            + s.floor * k.deep
            + s.pit * k.weight
            + self.band.at(p) * k.scale;
        (s.land + self.band.coast(dir), rest)
    }

    /// Returns the large height in metres above the sea in unit direction `dir`.
    pub fn large(&self, dir: DVec3) -> f32 {
        let (blend, rest) = self.parts(dir);
        shelf(blend, self.shore(), &self.scales) + rest
    }

    /// Returns what the large shape does to the local ground in unit direction `dir`.
    pub fn lift(&self, dir: DVec3) -> Lift {
        let level = self.large(dir);
        let bias = (level / FRINGE).clamp(-1.0, 1.0);
        Lift {
            bias,
            height: level - FRINGE * bias,
        }
    }

    /// Returns the offset on the crust blend that puts the low share of the planet at or under the sea:
    /// the datum of a large planet, measured once by sampling the sphere. The offset moves the shore
    /// across the continental margins and leaves every height in metres as it is, so the ocean floors stay
    /// deep and the continents stay low.
    fn shore(&self) -> f32 {
        *self.shore.get_or_init(|| {
            let parts: Vec<(f32, f32)> = sites().map(|dir| self.parts(dir)).collect();
            let low = |offset: f32| {
                let under =
                    |(blend, rest): &&(f32, f32)| shelf(*blend, offset, &self.scales) + rest <= 0.0;
                parts.iter().filter(under).count() as f64 / parts.len() as f64
            };
            shore(self.plates.interior().lowland, low)
        })
    }
}

/// Returns the height in metres of the crust at `blend` with the shore at `offset`, with `scales`: read
/// [`SHARPEN`] times steeper, the plain of the continent falls to the shelf break where blend and offset
/// sum to 0, then the slope goes down to the ocean floor. Past the offset that leaves no ocean floor, the
/// whole ground rises with it, see [`DRY`].
fn shelf(blend: f32, offset: f32, scales: &Scales) -> f32 {
    let land = ((blend + offset) * SHARPEN).clamp(-1.0, 1.0);
    let (plain, edge) = (PLAIN * scales.scale, SHELF * scales.scale);
    let crust = match land >= 0.0 {
        true => edge + (plain - edge) * land,
        false => edge + (-scales.deep - edge) * smooth(0.0, 1.0, -land),
    };
    crust + DRY.1 * scales.scale * (offset - DRY.0).max(0.0)
}
