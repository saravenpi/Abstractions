//! Aerial perspective: how much of each colour the air between the camera and a far thing lets
//! through, and how far the camera sees through it.
//!
//! One rule serves the ground, the props, the sea and the clouds. The gas of the air scatters blue
//! most, by the same optical depth [`DEPTH`] that reddens a low star, spread over the scale height
//! of the planet: on a planet like the Earth that is the measured Rayleigh scattering per metre.
//! Aerosols, the dust and the droplets of the low air, take every colour nearly alike and lie in a
//! thinner layer. What the air takes out of the light of a far thing it gives back as the colour
//! of the horizon, so far ridges fade into the sky step by step.

mod tune;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use self::tune::{AEROSOL, CONTRAST};
use self::tune::{AEROSOL_SCALE, DUST_AIR, FLAT, HOME_SCALE, LEFT};
use super::air::Air;
use super::scatter::DEPTH;
use super::vantage::Vantage;
use super::weather::{LocalWeather, WeatherLook};

/// Scale heights between the sea level and the top of the air, as the universe has it.
const TOP_SCALES: f32 = crate::universe::TOP_SCALES as f32;

/// What the air of one planet takes out of light per metre at sea level, under one weather.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Haze {
    /// Share of red, green and blue light the gas of the air scatters per metre at sea level.
    pub gas: Vec3,
    /// Share of red, green and blue light the aerosols take per metre at sea level.
    pub aerosol: Vec3,
    /// Height in metres over which the gas thins by a factor e. Zero without air.
    pub scale: f32,
}

impl Haze {
    /// The clear air of the home planet at the scale height of the Earth, for a camera that has
    /// no planet yet.
    pub const HOME: Self = Self {
        gas: Vec3::new(
            DEPTH.x / HOME_SCALE,
            DEPTH.y / HOME_SCALE,
            DEPTH.z / HOME_SCALE,
        ),
        aerosol: AEROSOL,
        scale: HOME_SCALE,
    };

    /// Returns the haze of `air` that ends `atmosphere` metres up, under the weather `look`:
    /// rain, dust and damp air thicken the aerosols by what they take from the sight. The gas thins
    /// with the pressure. The aerosols thin with its square root only, since thin air still carries
    /// dust and ice, and dusty air carries more: a thin dry air keeps some depth, a vacuum has none.
    pub fn of(air: &Air, atmosphere: f32, look: &WeatherLook) -> Self {
        let scale = atmosphere.max(0.0) / TOP_SCALES;
        let dense = air.share() * air.depth();
        if scale <= 0.0 || dense <= 0.0 {
            return Self {
                gas: Vec3::ZERO,
                aerosol: Vec3::ZERO,
                scale: 0.0,
            };
        }
        let load = air.share().sqrt() * air.depth() * (1.0 + DUST_AIR * air.dust());
        Self {
            gas: DEPTH * (dense / scale),
            aerosol: AEROSOL * (load / look.sight().max(1.0e-3)),
            scale,
        }
    }

    /// Returns the share of red, green and blue light taken per metre between the camera of
    /// `vantage` and a shell `radius` metres from the centre of the planet below, through `air`
    /// under the weather report `sky`: what fades a far cloud into the horizon.
    pub fn seen(vantage: &Vantage, air: &Air, sky: Option<&LocalWeather>, radius: f32) -> Vec3 {
        let haze = Self::of(air, vantage.atmosphere, &WeatherLook::of(sky));
        haze.toward(vantage.altitude, radius - vantage.radius)
    }

    /// Returns what the ground shader works the air out from at every point it draws, so a valley
    /// fills with haze and a peak stands clear of it: the gas per metre at sea level as a multiple
    /// of the optical depth [`DEPTH`], the aerosols as a multiple of [`AEROSOL`], and the scale
    /// height in metres. `aerial` in `daylight.wesl` reads them and holds the two constants under
    /// the same names.
    pub fn layers(&self) -> Vec3 {
        Vec3::new(self.gas.y / DEPTH.y, self.aerosol.y / AEROSOL.y, self.scale)
    }

    /// Returns the share of red, green and blue light the air takes per metre, on average, along a
    /// straight path between a point `from` metres above sea level and one `to` metres above it.
    pub fn toward(&self, from: f32, to: f32) -> Vec3 {
        if self.scale <= 0.0 {
            return Vec3::ZERO;
        }
        let gas = mean(from, to, self.scale);
        let low = mean(from, to, self.scale * AEROSOL_SCALE);
        self.gas * gas + self.aerosol * low
    }
}

/// Returns the mean density of a layer that thins by a factor e every `scale` metres, along a
/// straight path between the heights `from` and `to` in metres, as a share of its density at
/// sea level. A height below the sea counts as the sea. On flat ground this is exact whatever the
/// slant of the path, and from high above it leaves the path the depth of the whole layer.
pub fn mean(from: f32, to: f32, scale: f32) -> f32 {
    let (low, high) = (from.min(to).max(0.0) / scale, from.max(to).max(0.0) / scale);
    let span = high - low;
    if span < FLAT {
        return (-low).exp();
    }
    ((-low).exp() - (-high).exp()) / span
}

/// Returns the distance in metres at which air that takes `extinction` per metre leaves [`LEFT`]
/// of the colour it lets through best, and nothing beyond it shows. Without air it has no end.
pub fn sight(extinction: Vec3) -> f32 {
    let least = extinction.min_element();
    if least <= 0.0 {
        return f32::INFINITY;
    }
    -LEFT.ln() / least
}

/// Returns the share of red, green and blue light left after `distance` metres of air that
/// takes `extinction` per metre: what the fog of the camera and the shaders of the sea and the
/// clouds apply.
#[cfg(test)]
pub fn through(extinction: Vec3, distance: f32) -> Vec3 {
    (-extinction * distance).exp()
}
