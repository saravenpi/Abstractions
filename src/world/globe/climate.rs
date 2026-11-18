//! The climate on the globe: the place the climate field is read at, built from the region grids, and the
//! biome it gives on a sphere.

use std::cell::Cell;

use bevy::math::{DVec3, Vec2};

use super::Globe;
use super::atlas::STEPS;
use super::season::MEAN_SEASON;
use crate::climate::{self, Climate, Setting, Spot};
use crate::universe::{HOME_DAY_SECONDS, PlanetRecord};
use crate::world::biome::Biome;
use crate::world::islands::{gradient, sub_seed};

/// Metres across the patches of the noise that blurs biome borders.
const JITTER_SIZE: f64 = 90.0;

/// Continent measure deep inland: the climate reads the measure over this, about 1 deep inland.
pub const INLAND: f32 = 0.45;

/// A biome remembered for one planet fingerprint and one sphere point, bit for bit.
type Known = Option<(u64, [u64; 3], Biome)>;

thread_local! {
    /// The last biomes each thread found on the sphere: a pond candidate asks for the biome at its centre
    /// for every ground sample around it.
    static KNOWN: Cell<[Known; 4]> = const { Cell::new([None; 4]) };
}

/// Returns the climate setting of a planet built without a spec: the home record and the home day.
pub fn setting(seed: u64, radius: f64, gravity: f64) -> Setting {
    let record = PlanetRecord::TEMPERATE;
    Setting {
        temperature: record.temperature as f32,
        pressure: record.pressure as f32,
        ocean: record.ocean as f32,
        tilt: record.tilt as f32,
        day_seconds: HOME_DAY_SECONDS as f32,
        gravity: gravity as f32,
        radius: radius as f32,
        seed,
        near: None,
    }
}

/// Returns `setting` with the air, heat, water and tilt of `record`.
pub fn recorded(mut setting: Setting, record: &PlanetRecord) -> Setting {
    setting.temperature = record.temperature as f32;
    setting.pressure = record.pressure as f32;
    setting.ocean = record.ocean as f32;
    setting.tilt = record.tilt as f32;
    setting
}

/// Returns the unit directions east and north at unit direction `dir`, the spin axis being +Y; at a pole,
/// where east is undefined, east is +X.
pub fn frame(dir: DVec3) -> (DVec3, DVec3) {
    let east = DVec3::Y.cross(dir).try_normalize().unwrap_or(DVec3::X);
    (east, dir.cross(east))
}

impl Globe {
    /// Returns the climate in unit direction `dir` at the current season, see [`Globe::set_season`].
    pub fn climate(&self, dir: DVec3) -> Climate {
        self.climate_in(dir, self.season())
    }

    /// Returns the climate in unit direction `dir` at `season`.
    pub(super) fn climate_in(&self, dir: DVec3, season: f32) -> Climate {
        climate::at(&self.setting, &self.spot(dir), season)
    }

    /// Returns the place in unit direction `dir` of the planet frame as the climate reads it, from the
    /// region grid: its height above the liquid, or above the datum on a dry world, its inland measure and
    /// its rise per metre towards the east and the north. Every reader of the climate on the ground uses this.
    pub fn spot(&self, dir: DVec3) -> Spot {
        let (sample, face) = self.with_region(dir, |cell, sample| (sample, cell.face));
        let [normal, a, b] = face.axes();
        let step = 2.0 / f64::from(self.cells() * STEPS as i32) * self.radius() * dir.dot(normal);
        let per = |axis: DVec3| (axis - dir * axis.dot(dir)) * step;
        let (along_a, along_b) = (per(a), per(b));
        let rise = along_a * (f64::from(sample.rise[0]) / along_a.length_squared())
            + along_b * (f64::from(sample.rise[1]) / along_b.length_squared());
        let (east, north) = frame(dir);
        let altitude = match self.has_sea() {
            true => sample.height.max(0.0),
            false => sample.height,
        };
        Spot {
            dir,
            altitude,
            inland: sample.inland / INLAND,
            slope: Vec2::new(rise.dot(east) as f32, rise.dot(north) as f32),
        }
    }

    /// Returns the biome of the annual mean climate at sphere point `p`, its borders blurred by noise that
    /// only jitters the climate, never picks the biome.
    pub(super) fn biome_at(&self, p: DVec3) -> Biome {
        let key = (self.atlas.print, p.to_array().map(f64::to_bits));
        let mut known = KNOWN.get();
        if let Some((_, _, biome)) = known.iter().flatten().find(|k| (k.0, k.1) == key) {
            return *biome;
        }
        let biome = self.biome_found(p);
        known.rotate_right(1);
        known[0] = Some((key.0, key.1, biome));
        KNOWN.set(known);
        biome
    }

    /// Returns the biome of the annual mean climate at sphere point `p`, found anew.
    fn biome_found(&self, p: DVec3) -> Biome {
        let climate = self.climate_in(p.normalize(), MEAN_SEASON);
        let jitter = gradient(sub_seed(self.seed, 18), (p / JITTER_SIZE).to_array());
        climate::biome(&climate, jitter as f32)
    }
}
