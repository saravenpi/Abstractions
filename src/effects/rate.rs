//! How fast one plant sheds, from its season, its water, the wind, the rain and the air: leaves, petals
//! and pollen specks per second per crown.
//!
//! Drag on a leaf grows with the density of the air times the square of the wind speed, so a leaf lets go
//! when that pressure passes what its stalk holds, and the rate grows with the square of the excess.

use super::sky::Sky;
use super::tune::{self, Grip};
use crate::life::flora::Stage;
use crate::world::canopy::Shed;

/// The air around a plant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Air {
    /// Wind speed in metres per second.
    pub speed: f32,
    /// How thick the air is against the home planet, 0 without air.
    pub thickness: f32,
    /// How hard it rains, from 0 to 1.
    pub rain: f32,
}

impl Air {
    /// Returns the air of a place where the wind blows at `speed` metres per second in air of `thickness`
    /// under `sky`.
    pub fn around(speed: f32, thickness: f32, sky: &Sky) -> Self {
        Self {
            speed,
            thickness,
            rain: sky.rain,
        }
    }

    /// Returns the dynamic pressure of the wind in pascals: half the density times the speed squared.
    fn pressure(&self) -> f32 {
        0.5 * tune::DENSITY * self.thickness * self.speed * self.speed
    }

    /// Returns how far the wind is past `hold` pascals, as a share of it, squared: 0 under it.
    fn gale(&self, hold: f32) -> f32 {
        (self.pressure() / hold - 1.0).max(0.0).powi(2)
    }
}

/// Returns how the plant of `shed` holds its leaves in its season.
fn grip(shed: &Shed) -> Grip {
    match (shed.deciduous, shed.stage) {
        (false, _) => tune::EVERGREEN,
        (true, Stage::Ever) => tune::RENEWING,
        (true, Stage::Bloom) => tune::SPRING,
        (true, Stage::Green) => tune::SUMMER,
        (true, Stage::Fruit) => tune::AUTUMN,
        (true, Stage::Dormant) => tune::WINTER,
    }
}

/// Returns the leaves per second the plant of `shed` loses in `air`: nothing without leaves or without
/// air, a trickle in still air, and in a gust or a shower the more the looser its leaves hang.
pub fn leaves(shed: &Shed, air: &Air) -> f32 {
    if shed.leaf.is_none() || air.thickness <= 0.0 {
        return 0.0;
    }
    let grip = grip(shed);
    let torn = tune::WIND_GAIN * air.gale(tune::HOLD_LEAF) * (1.0 + air.rain);
    let knocked = tune::RAIN_GAIN * air.rain;
    let thirst = 1.0 + tune::STRESS_GAIN * shed.stress;
    ((grip.calm + grip.loose * (torn + knocked)) * thirst).min(tune::MOST)
}

/// Returns the petals per second the plant of `shed` loses in `air`: nothing out of flower.
pub fn petals(shed: &Shed, air: &Air) -> f32 {
    if shed.flower.is_none() || air.thickness <= 0.0 {
        return 0.0;
    }
    let torn = tune::PETAL_WIND * air.gale(tune::HOLD_PETAL);
    let rained = tune::PETAL_RAIN * air.rain;
    (tune::PETAL_CALM + torn + rained).min(tune::MOST)
}

/// Returns the pollen specks per second the plant of `shed` lets go in `air`: only in flower, only
/// when it is dry, and the more the wind lifts it.
pub fn pollen(shed: &Shed, air: &Air) -> f32 {
    let (low, high) = tune::POLLEN_WIND;
    let lift = ((air.speed - low) / (high - low)).clamp(0.0, 1.0);
    let dry = air.rain < tune::POLLEN_RAIN && air.thickness > 0.0;
    match shed.flower.is_some() && dry {
        true => tune::POLLEN * lift * lift * (3.0 - 2.0 * lift),
        false => 0.0,
    }
}

#[cfg(test)]
#[path = "tests_rate.rs"]
mod tests;
