//! The hot skin of the nose and the leading edges: a temperature that the heat flux of the air
//! raises and radiation lowers, so the skin warms over about a second in an entry and, once the
//! air is gone, fades from sight within seconds by day, a faint red edge lingering a little
//! longer at night.
//!
//! `C dT/dt = q - e sigma T^4`, with `C` the heat capacity of the skin per square metre, `q` the
//! heat flux of the air and `e sigma T^4` what the skin radiates away. Radiation cools a hot skin
//! fast and a warm one slowly.

use super::temperature;
use super::tune::{
    AMBIENT, EMISSIVITY, HEATING, RED, SKIN_CAPACITY, SKIN_DECADES, SKIN_GLOW, SKIN_STEP,
    STEFAN_BOLTZMANN,
};
use crate::render::celestial::smoothstep;

/// Returns the temperature in kelvin of a skin at `kelvin` after `dt` seconds under the heat
/// flux `flux` in watts per square metre. A skin at 0 is cold and dark: it starts from
/// [`AMBIENT`] once the air heats it. It never passes the temperature at which it would radiate
/// all of the flux away, and once it is too cool to glow with no flux on it, it is 0 again.
pub fn warmed(kelvin: f32, flux: f32, dt: f32) -> f32 {
    if kelvin <= 0.0 && flux <= 0.0 {
        return 0.0;
    }
    let target = (temperature(f64::from(flux)) as f32).max(AMBIENT);
    let steps = (dt.max(0.0) / SKIN_STEP).ceil().clamp(1.0, 40.0);
    let h = dt.max(0.0) / steps;
    let sigma = (EMISSIVITY * STEFAN_BOLTZMANN) as f32;
    let mut now = kelvin.max(AMBIENT);
    for _ in 0..steps as u32 {
        let next = now + (flux - sigma * now.powi(4)) / SKIN_CAPACITY * h;
        now = if now < target {
            next.min(target)
        } else {
            next.max(target)
        };
    }
    match flux <= 0.0 && now < SKIN_GLOW.0 {
        true => 0.0,
        false => now,
    }
}

/// Returns how strongly a skin at `kelvin` glows, from 0 to 1 at the full glow and above: its
/// radiance in the red of [`RED`] against the one at the full glow, by Planck's law, taken on a
/// scale of decades as the eye takes light, so the glow fades in smoothly over the last
/// [`SKIN_DECADES`] decades. Radiance falls steeply as the skin cools, so a skin that is still
/// hot to the touch is soon dark to the eye.
pub fn glowing(kelvin: f32) -> f32 {
    if kelvin < SKIN_GLOW.0 {
        return 0.0;
    }
    let band = 1.438_777e-2 / RED;
    let full = (band / SKIN_GLOW.1).exp() - 1.0;
    let decades = (full / ((band / kelvin).exp() - 1.0)).log10();
    smoothstep(-SKIN_DECADES, 0.0, decades)
}

/// Returns the temperature in kelvin of the skin of a craft whose records carry only the heat
/// share `heat`: the flux is read back along the logarithm of the bounds of [`HEATING`], and
/// the skin is taken at the temperature that radiates it away. Nothing for no heat.
pub fn kelvin_of(heat: f32) -> f32 {
    if heat <= 0.0 {
        return 0.0;
    }
    let (low, full) = HEATING;
    let log = low.log10() + f64::from(heat.min(1.0)) * (full.log10() - low.log10());
    temperature(10f64.powf(log)) as f32
}
