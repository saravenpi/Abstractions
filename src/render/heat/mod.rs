//! The heat of an atmospheric entry: how hard the air heats the nose of a craft, from the
//! density of the air of the planet and the speed through it, the colour that heat glows in the
//! gas of that air and the colour of the hot skin itself.
//!
//! The heating of the nose is the convective rate of Sutton and Graves,
//! `q = k * sqrt(rho / r) * v^3`, with `rho` the density of the air, `v` the speed through it
//! and `r` the radius of the nose (<https://en.wikipedia.org/wiki/Atmospheric_entry>). It is
//! the same on the way down and on the way up: only the air and the speed count. The ship and
//! the gear write what they fly through into [`Heat`]; the sheath, the hull light, the cabin,
//! the shake and the roar read it back.

mod gas;
mod sheath;
mod state;
mod tune;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_path;
#[cfg(test)]
mod tests_skin;

use bevy::prelude::*;

pub use gas::Gas;
pub use sheath::{SheathMaterial, SheathUniform};
pub use state::{Craft, Heat, HeatSet};
pub use tune::{MIN_SPEED, SHEATH_FROM};

use super::celestial::smoothstep;
use gas::incandescence;

/// How hot the nose of a craft glows and how hard the air presses on it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Glow {
    /// Strength of the glow from 0 to 1: 0 below the heating that shows.
    pub heat: f32,
    /// Strength of the roar and the shake from 0 to 1, from the dynamic pressure.
    pub pressure: f32,
    /// Linear colour of the glowing gas of the shock.
    pub color: Vec3,
    /// Linear colour of the hot skin of the nose and the leading edges, from its temperature.
    pub skin: Vec3,
}

impl Glow {
    /// No glow, no pressure.
    pub const NONE: Self = Self {
        heat: 0.0,
        pressure: 0.0,
        color: Vec3::ZERO,
        skin: Vec3::ZERO,
    };

    /// Returns the glow `dt` seconds after this one when the craft now makes `next`: the
    /// strength rises and falls at the rates of [`tune::EASE`], and the colours follow `next`
    /// while there is a glow to colour.
    pub fn follow(self, next: Self, dt: f32) -> Self {
        let rate = |now: f32, to: f32| {
            let speed = if to > now { tune::EASE.0 } else { tune::EASE.1 };
            let eased = now + (to - now) * (1.0 - (-speed * dt).exp());
            if to == 0.0 && eased < 1.0e-3 {
                0.0
            } else {
                eased
            }
        };
        let hue = 1.0 - (-tune::HUE_RATE * dt).exp();
        let tint = |now: Vec3, to: Vec3| match (self.heat > 0.0, next.heat > 0.0) {
            (_, false) => now,
            (false, true) => to,
            (true, true) => now.lerp(to, hue),
        };
        Self {
            heat: rate(self.heat, next.heat),
            pressure: rate(self.pressure, next.pressure),
            color: tint(self.color, next.color),
            skin: tint(self.skin, next.skin),
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

/// Returns the linear colour of the hot skin of a nose that makes the heat share `heat`, for a
/// craft whose records carry only that share: the flux is read back along the logarithm of the
/// bounds of [`tune::HEATING`], and its [`temperature`] gives the colour.
pub fn skin(heat: f32) -> Vec3 {
    let (low, full) = tune::HEATING;
    let log = low.log10() + f64::from(heat.clamp(0.0, 1.0)) * (full.log10() - low.log10());
    incandescence(temperature(10f64.powf(log)) as f32)
}

/// Returns the glow of a craft flying at `speed` metres per second through air of `density`
/// kilograms per cubic metre made of `gas`, whichever way it flies.
///
/// The strength comes from the flux of [`flux`], the colour of the shock from the gas, the
/// colour of the skin from the [`temperature`] the flux gives, and the pressure is the dynamic
/// pressure `rho * v^2 / 2` between the bounds of [`tune::PRESSURE`].
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
        skin: incandescence(temperature(heating) as f32),
    }
}

/// Registers the heat, its sheath and the shader of the sheath.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((state::plugin, sheath::plugin));
    super::warm_material::<sheath::SheathMaterial>(app);
}
