//! Landforms: what each class of planet adds to the shape of its ground on the sphere, before any flow or
//! pond is carved. The legacy plane has none.

use bevy::math::{DVec3, Vec2};

use super::Globe;
use super::climate::INLAND;
use super::season::MEAN_SEASON;
use crate::climate::{self, Spot};
use crate::universe::PlanetClass;
use crate::world::SEA_LEVEL;
use crate::world::islands::{gradient, smooth, sub_seed};

/// Metres between dune crests across them, and along them, which run north to south.
const DUNE_SIZE: (f64, f64) = (55.0, 330.0);

/// Lowest and highest dune crest in metres.
const DUNE_HEIGHT: (f32, f32) = (3.0, 12.0);

/// Metres across the patches of taller and lower dunes.
const DUNE_FIELD: f64 = 1100.0;

/// Metres across the noise whose zero lines are the glacier valleys.
const VALLEY_SIZE: f64 = 700.0;

/// Depth of a glacier valley in metres.
const VALLEY_DEPTH: f32 = 16.0;

/// Kelvin under which glaciers fill the valleys, fully from 4 K lower.
const GLACIAL: f32 = 260.0;

/// The landform a class of planet adds to its ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landform {
    /// Nothing: the ground of the generator as it is.
    Plain,
    /// Sand dunes across the wind, in the low ground of a desert.
    Dunes,
    /// Impact craters over a world without air.
    Craters,
    /// Glacier valleys where it is cold, and the frozen sea walkable at the datum.
    Glaciers,
}

impl Landform {
    /// Returns the landform of a class.
    pub fn of(class: PlanetClass) -> Self {
        match class {
            PlanetClass::Desert => Landform::Dunes,
            PlanetClass::Barren => Landform::Craters,
            PlanetClass::Ice => Landform::Glaciers,
            _ => Landform::Plain,
        }
    }
}

/// Returns the dune height at sphere point `p` of planet `seed`, over ground `height` high: ridges whose
/// crests run along the spin axis, across the east and west winds of the climate bands, fading out above
/// 15 m so the dunes fill the low ground.
pub fn dunes(seed: u64, p: DVec3, height: f32) -> f32 {
    let low = smooth(40.0, 15.0, height);
    if low <= 0.0 {
        return 0.0;
    }
    let q = [p.x / DUNE_SIZE.0, p.y / DUNE_SIZE.1, p.z / DUNE_SIZE.0];
    let crest = 1.0 - gradient(sub_seed(seed, 21), q).abs() as f32;
    let field = gradient(sub_seed(seed, 22), (p / DUNE_FIELD).to_array()) as f32;
    let tall = DUNE_HEIGHT.0 + (DUNE_HEIGHT.1 - DUNE_HEIGHT.0) * smooth(-0.3, 0.4, field);
    low * tall * crest * crest
}

impl Globe {
    /// Returns the landform of the planet.
    pub(super) fn landform(&self) -> Landform {
        Landform::of(self.record.class)
    }

    /// Returns the ground height at sphere point `p` before flows and ponds: the generator's ground moved by
    /// the datum, `base` being its inland measure, with the landform of the class added.
    pub(super) fn shaped(&self, p: DVec3) -> f32 {
        let (raw, base) = self.shape.measure(p.to_array(), &self.climate);
        let height = raw - self.datum;
        match self.landform() {
            Landform::Plain => height,
            Landform::Dunes => height + dunes(self.seed, p, height),
            Landform::Craters => height + self.pitted(p),
            Landform::Glaciers => self.glaciated(p, height, base),
        }
    }

    /// Returns the ground `height` at sphere point `p` with its glacier valleys, where the annual mean is
    /// under 260 K, and on a world without a sea the frozen sea that stands at the datum.
    fn glaciated(&self, p: DVec3, height: f32, base: f32) -> f32 {
        let floor = match self.has_sea() {
            true => f32::NEG_INFINITY,
            false => SEA_LEVEL,
        };
        let land = smooth(2.0, 12.0, height);
        let valley = gradient(sub_seed(self.seed, 20), (p / VALLEY_SIZE).to_array()).abs() as f32;
        let trough = (1.0 - smooth(0.08, 0.22, valley)) * land;
        if trough <= 0.0 {
            return height.max(floor);
        }
        let spot = Spot {
            dir: p.normalize(),
            altitude: height,
            inland: base / INLAND,
            slope: Vec2::ZERO,
        };
        let cold = smooth(
            GLACIAL + 2.0,
            GLACIAL - 4.0,
            climate::at(&self.setting, &spot, MEAN_SEASON).temperature,
        );
        (height - VALLEY_DEPTH * trough * cold).max(floor)
    }

    /// Returns the ground height at sphere point `p` before flows and ponds, and its inland measure.
    pub(super) fn relief(&self, p: DVec3) -> (f32, f32) {
        (self.shaped(p), self.shape.base(p.to_array()))
    }
}
