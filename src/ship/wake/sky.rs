//! The air of a planet at the height of a hull, as its condensation trails read it: how cold it
//! is, at what pressure, how fast sound runs in it and how far under the cloud base the hull is.

use crate::speed::{sound_speed, temperature_at};
use crate::universe::{PlanetRecord, TOP_SCALES, heights};

/// Pressure at sea level of the home planet, in pascals.
const SEA_PRESSURE: f64 = 101_325.0;

/// The air at one height.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Column {
    /// Temperature in kelvin.
    pub cold: f32,
    /// Pressure in pascals, zero above the top of the air.
    pub pressure: f32,
    /// Speed of sound in metres per second.
    pub sound: f32,
    /// Metres from the hull up to the cloud base, negative inside or above the clouds.
    pub below: f32,
}

/// Returns the air `height` metres above the sea of a planet of `record`, with the surface
/// `gravity` in metres per second squared and the `radius` in metres.
///
/// Rising air cools as the speed module says up to the top of the weather layer and keeps its
/// temperature above it, and the pressure falls by a factor e over each scale height.
pub fn column(record: &PlanetRecord, (gravity, radius): (f64, f64), height: f64) -> Column {
    let [top, base, weather] = heights(record, gravity, radius);
    let scale = top / TOP_SCALES;
    let cold = temperature_at(
        record.temperature,
        record.mix.kappa(),
        (scale, weather),
        height,
    );
    let pressure = match height < top {
        true => SEA_PRESSURE * record.pressure * (-height.max(0.0) / scale).exp(),
        false => 0.0,
    };
    Column {
        cold: cold as f32,
        pressure: pressure as f32,
        sound: sound_speed(cold, &record.mix),
        below: (base - height) as f32,
    }
}
