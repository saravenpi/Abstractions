//! The heat of an atmospheric entry: how hard the air heats the nose of a craft, from the
//! density of the air of the planet and the speed through it, the colour that heat glows in the
//! gas of that air and the colour of the hot skin itself.
//!
//! The heating of the nose is the convective rate of Sutton and Graves,
//! `q = k * sqrt(rho / r) * v^3`, with `rho` the density of the air, `v` the speed through it
//! and `r` the radius of the nose (<https://en.wikipedia.org/wiki/Atmospheric_entry>). It is
//! the same on the way down and on the way up: only the air and the speed count. Two things
//! glow: the plasma of the shock, which follows the flux of this moment and so fades within
//! seconds as the air thins, and the skin of the nose, which the flux heats over seconds and
//! radiation cools over tens of seconds after, see `skin`. The ship and the gear write what they
//! fly through into [`Heat`]; the sheath, the cabin, the shake and the roar read it back.

mod gas;
mod sheath;
mod skin;
mod state;
mod tune;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cool;
#[cfg(test)]
mod tests_path;
#[cfg(test)]
mod tests_skin;

use bevy::prelude::*;

pub use gas::Gas;
#[cfg(test)]
pub use gas::incandescence;
pub use sheath::{SheathMaterial, SheathUniform};
pub use skin::kelvin_of;
pub use state::{Craft, Heat};
pub use tune::MIN_SPEED;

use super::celestial::smoothstep;

/// How hot the nose of a craft glows and how hard the air presses on it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Glow {
    /// Strength of the glow of the plasma of the shock from 0 to 1: 0 below the heating that
    /// shows.
    pub heat: f32,
    /// Strength of the roar and the shake from 0 to 1, from the dynamic pressure.
    pub pressure: f32,
    /// Linear colour of the glowing gas of the shock.
    pub color: Vec3,
    /// Heat flux of the air on the nose in watts per square metre.
    pub flux: f32,
    /// Temperature of the skin of the nose and the leading edges in kelvin, 0 while it is cold
    /// and dark.
    pub kelvin: f32,
    /// How strongly the hot skin glows as it is drawn, from 0 to 1: its glow at its temperature,
    /// faded in no faster than [`tune::SKIN_RISE`].
    pub shine: f32,
}

impl Glow {
    /// No glow, no pressure, no heat.
    pub const NONE: Self = Self {
        heat: 0.0,
        pressure: 0.0,
        color: Vec3::ZERO,
        flux: 0.0,
        kelvin: 0.0,
        shine: 0.0,
    };

    /// Returns the glow `dt` seconds after this one when the craft now makes `next`: the plasma
    /// and the pressure follow `next` no faster than [`tune::PLASMA_RATE`], the colour of the gas
    /// follows it while there is a glow to colour, the skin warms or cools from its own
    /// temperature under the flux of `next`, and its glow fades in no faster than
    /// [`tune::SKIN_RISE`].
    pub fn follow(self, next: Self, dt: f32) -> Self {
        let (up, down) = tune::PLASMA_RATE;
        let rate = |now: f32, to: f32| now + (to - now).clamp(-down * dt, up * dt);
        let kelvin = skin::warmed(self.kelvin, next.flux, dt);
        let lit = skin::glowing(kelvin);
        let hue = 1.0 - (-tune::HUE_RATE * dt).exp();
        let color = match (self.heat > 0.0, next.heat > 0.0) {
            (_, false) => self.color,
            (false, true) => next.color,
            (true, true) => self.color.lerp(next.color, hue),
        };
        Self {
            heat: rate(self.heat, next.heat),
            pressure: rate(self.pressure, next.pressure),
            color,
            flux: next.flux,
            kelvin,
            shine: lit.min(self.shine + tune::SKIN_RISE * dt),
        }
    }
}

/// Returns the heat flux at the nose in watts per square metre for air of `density` kilograms
/// per cubic metre and a speed of `speed` metres per second through it.
///
/// It grows as the root of the density and as the cube of the speed: twice the speed gives
/// eight times the heat. No air gives none.
pub fn flux(density: f64, speed: f64) -> f64 {
    let k = tune::SUTTON_GRAVES / tune::NOSE_RADIUS.sqrt();
    k * density.max(0.0).sqrt() * speed.max(0.0).powi(3)
}

/// Returns the temperature in kelvin of a nose that radiates away the heat `flux` in watts per
/// square metre: `q = e * sigma * T^4`.
pub fn temperature(flux: f64) -> f64 {
    (flux.max(0.0) / (tune::EMISSIVITY * tune::STEFAN_BOLTZMANN)).powf(0.25)
}

/// Returns the highest speed in metres per second at which the nose stays dark in air of
/// `density` kilograms per cubic metre: the speed at which [`flux`] reaches the first bound of
/// [`tune::HEATING`]. Without air there is no bound.
pub fn cool_speed(density: f64) -> f64 {
    if density <= 0.0 {
        return f64::INFINITY;
    }
    let k = tune::SUTTON_GRAVES / tune::NOSE_RADIUS.sqrt();
    (tune::HEATING.0 / (k * density.sqrt())).cbrt()
}

/// Returns the share of the glow a heat flux of `flux` watts per square metre makes, from 0
/// to 1, along the logarithm of the flux between the bounds of [`tune::HEATING`].
fn share(flux: f64) -> f32 {
    if flux <= 0.0 {
        return 0.0;
    }
    let (low, full) = tune::HEATING;
    smoothstep(low.log10() as f32, full.log10() as f32, flux.log10() as f32)
}

/// Returns the glow of a craft flying at `speed` metres per second through air of `density`
/// kilograms per cubic metre made of `gas`, whichever way it flies.
///
/// The strength comes from the flux of [`flux`], a smooth function of it that reaches 0
/// continuously, the colour of the shock from the gas, the temperature of the skin is the one
/// that radiates the flux away, which a skin only reaches over time (see [`Glow::follow`]), and
/// the pressure is the dynamic pressure `rho * v^2 / 2` between the bounds of [`tune::PRESSURE`].
pub fn entry(density: f64, speed: f64, gas: Gas) -> Glow {
    let (density, speed) = (density.max(0.0), speed.max(0.0));
    let heating = flux(density, speed);
    let heat = share(heating);
    let (from, to) = tune::PRESSURE;
    let load = 0.5 * density * speed * speed;
    Glow {
        heat,
        pressure: smoothstep(from as f32, to as f32, load as f32),
        color: gas.color(heat),
        flux: heating as f32,
        kelvin: temperature(heating) as f32,
        shine: skin::glowing(temperature(heating) as f32),
    }
}

/// Registers the heat, its sheath and the shader of the sheath.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((state::plugin, sheath::plugin));
    super::warm_material::<sheath::SheathMaterial>(app);
}
