//! The heat of an atmospheric entry: how hard the air heats the nose of a craft, from the
//! density of the air of the planet and the speed through it, and the colour that heat glows
//! in the gas of that air.
//!
//! The heating of the nose is the convective rate of Sutton and Graves,
//! `q = k * sqrt(rho / r) * v^3`, with `rho` the density of the air, `v` the speed through it
//! and `r` the radius of the nose (<https://en.wikipedia.org/wiki/Atmospheric_entry>). The
//! ship and the gear write what they fly through into [`Heat`]; the view, the sheath, the shake
//! and the roar read it back.

mod gas;
mod sheath;
mod state;
mod tune;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use gas::Gas;
pub use sheath::{SheathMaterial, SheathUniform};
pub use state::{Craft, Heat, HeatSet};
pub use tune::{MIN_SPEED, SHEATH_FROM, SHEATH_REACH};

use super::celestial::smoothstep;

/// How hot the nose of a craft glows and how hard the air presses on it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Glow {
    /// Strength of the glow from 0 to 1: 0 below the heating that shows.
    pub heat: f32,
    /// Strength of the roar and the shake from 0 to 1, from the dynamic pressure.
    pub pressure: f32,
    /// Linear colour of the glow.
    pub color: Vec3,
}

impl Glow {
    /// No glow, no pressure.
    pub const NONE: Self = Self {
        heat: 0.0,
        pressure: 0.0,
        color: Vec3::ZERO,
    };

    /// Returns this glow with the strength of the entry cut to a `share` of itself.
    pub fn scaled(self, share: f32) -> Self {
        Self {
            heat: self.heat * share,
            pressure: self.pressure * share,
            ..self
        }
    }

    /// Returns the glow `dt` seconds after this one when the craft now makes `next`: the
    /// strength rises and falls at the rates of [`tune::EASE`], and the colour follows `next`
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
        let color = match (self.heat > 0.0, next.heat > 0.0) {
            (_, false) => self.color,
            (false, true) => next.color,
            (true, true) => self.color.lerp(next.color, hue),
        };
        Self {
            heat: rate(self.heat, next.heat),
            pressure: rate(self.pressure, next.pressure),
            color,
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

/// Returns the glow of a craft flying at `speed` metres per second through air of `density`
/// kilograms per cubic metre made of `gas`.
///
/// The strength is the flux of [`flux`] between the bounds of [`tune::HEATING`], and the
/// pressure is the dynamic pressure `rho * v^2 / 2` between those of [`tune::PRESSURE`].
pub fn entry(density: f64, speed: f64, gas: Gas) -> Glow {
    let (density, speed) = (density.max(0.0), speed.max(0.0));
    let (low, full) = tune::HEATING;
    let heat = smoothstep(low as f32, full as f32, flux(density, speed) as f32);
    let (from, to) = tune::PRESSURE;
    let load = 0.5 * density * speed * speed;
    Glow {
        heat,
        pressure: smoothstep(from as f32, to as f32, load as f32),
        color: gas.color(heat),
    }
}

/// Returns how much of a glow shows for a fall of `sink` metres per second toward the ground:
/// nothing in level flight and in a landing approach, all of it in a dive.
pub fn descent(sink: f32) -> f32 {
    smoothstep(tune::SINK.0, tune::SINK.1, sink)
}

/// Registers the heat, its sheath and the shader of the sheath.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((state::plugin, sheath::plugin));
    super::warm_material::<sheath::SheathMaterial>(app);
}
