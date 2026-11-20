//! Relief from physics: how high the mountains and how deep the oceans of a planet can be for its size and
//! its pull, and the large shape of the ground those heights scale.

use std::sync::Arc;

use bevy::math::DVec3;

use super::Globe;
use super::datum::datum;
use super::elevation::tune::{DEPTH, LOCAL, PEAK, SLOPE, STRENGTH};
use super::elevation::{Elevation, Lift};
use super::plates::{self, Interior, RANGE};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::islands::smooth;

/// The heights a planet can carry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scales {
    /// How much the plates count, see [`weight`]: the factor on what keeps its real size on a small
    /// planet, such as the depth of an impact basin.
    pub weight: f32,
    /// Factor on every height and length given in metres for a full-size planet: 0 on a world too small
    /// for plates, 1 from about the size of the Earth.
    pub scale: f32,
    /// Summit of the highest range in metres above the plain.
    pub peak: f32,
    /// Depth of the ocean floor in metres under the sea.
    pub deep: f32,
}

impl Scales {
    /// Returns the heights of a planet of `radius` metres and surface `gravity` in metres per second
    /// squared, whose low ground covers the share `lowland`.
    ///
    /// The highest mountain is the one the rock can carry, [`STRENGTH`] over the gravity within [`PEAK`].
    /// The ocean is 3 to 6 km deep, deeper the more of the planet it covers. Both shrink on a planet whose
    /// ranges are too narrow to stand that high at [`SLOPE`], and vanish by [`weight`].
    pub fn of(radius: f64, gravity: f64, lowland: f64) -> Self {
        let fit = (SLOPE * RANGE * radius / PEAK.1).min(1.0);
        let weight = weight(radius);
        let scale = fit * f64::from(weight);
        Self {
            weight,
            scale: scale as f32,
            peak: ((STRENGTH / gravity).clamp(PEAK.0, PEAK.1) * scale) as f32,
            deep: ((DEPTH.0 + (DEPTH.1 - DEPTH.0) * lowland.clamp(0.0, 1.0)) * scale) as f32,
        }
    }
}

/// Returns how much the plates count on a planet of `radius` metres: nothing while its ranges would be
/// narrower than the hills of the local ground, everything from [`LOCAL`]'s second width on. Exactly 0 up
/// to a radius of 50 km, and 1 from 400 km.
pub fn weight(radius: f64) -> f32 {
    smooth(LOCAL.0, LOCAL.1, (RANGE * radius) as f32)
}

/// The relief of one planet: where its sea stands in the local ground, and its large shape.
#[derive(Clone)]
pub struct Relief {
    /// Raw height of the sea level in the local ground, in metres: what every local height is measured
    /// from.
    pub datum: f32,
    /// Age of the planet in billions of years when one was given.
    pub age: Option<f64>,
    /// The large shape of the ground, or nothing on a world the local ground shapes alone and on one
    /// without ground.
    high: Option<Arc<Elevation>>,
}

impl Relief {
    /// Returns the relief of planet `seed` of radius and surface gravity `body`, with `record`, `age`
    /// billions of years old or as old as its seed says.
    ///
    /// The sea height of the local ground, from [`datum`], fades out as the plates fade in: on a large
    /// planet the shore is measured on the large shape instead and the local ground keeps its coast at 0.
    pub fn of(seed: u64, body: [f64; 2], record: &PlanetRecord, age: Option<f64>) -> Self {
        let weight = weight(body[0]);
        let grounded = weight > 0.0 && record.class != PlanetClass::GasGiant;
        let high = grounded.then(|| {
            let years = age.unwrap_or_else(|| plates::age(seed));
            let interior = Interior::of(seed, body, record, years);
            let scales = Scales::of(body[0], body[1], interior.lowland);
            Arc::new(Elevation::new(seed, body[0], scales, interior))
        });
        Self {
            datum: datum(record.ocean).unwrap_or(0.0) * (1.0 - weight),
            age,
            high,
        }
    }

    /// Returns the large shape of the ground, where the planet has one.
    pub fn high(&self) -> Option<&Elevation> {
        self.high.as_deref()
    }

    /// Returns what the large shape does to the local ground in unit direction `dir`, or nothing on a
    /// world the local ground shapes alone.
    pub fn lift(&self, dir: DVec3) -> Option<Lift> {
        self.high.as_ref().map(|high| high.lift(dir))
    }
}

impl Globe {
    /// Returns this globe as a planet `gyr` billions of years old. Without it the age follows the seed.
    ///
    /// The age decides how much inner heat is left: whether the plates still move, whether volcanoes
    /// grow, and how many impact basins the surface has kept.
    pub fn aged(mut self, gyr: f64) -> Self {
        self.relief = Relief::of(
            self.seed,
            [self.radius, self.gravity],
            &self.record,
            Some(gyr),
        );
        self.charted()
    }

    /// Returns what the plates say about unit direction `dir`, or nothing on a planet too small for
    /// plates and on one without ground.
    pub fn plate(&self, dir: DVec3) -> Option<plates::Sample> {
        self.relief.high().map(|high| high.plates().sample(dir))
    }

    /// Returns the large height of the ground in metres above the sea in unit direction `dir`, without
    /// the local ground: what a reader far too coarse for hills needs. Nothing where [`Globe::plate`]
    /// gives nothing.
    pub fn large_height(&self, dir: DVec3) -> Option<f32> {
        self.relief.high().map(|high| high.large(dir))
    }
}
