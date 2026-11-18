//! The air of one planet as the sky draws it: how thick it is, what it does to colours and how
//! much cloud it carries. All of it is read from the planet record, and the home planet gives
//! the values the sky was first tuned for.

use bevy::prelude::*;

use super::scatter::{luminance, peak};
use crate::universe::PlanetRecord;

/// Pressure under which a planet has no air to speak of, the home planet having 1.
const VACUUM: f32 = 0.01;

/// How much thicker than the home air the air can get to the eye, however high the pressure:
/// the depth saturates at one plus this, so a crushing sky stays readable.
const THICKEN: f32 = 1.5;

/// How fast a coloured air tint takes the sky over from the blue of clean air.
const DUST: f32 = 2.5;

/// Brightness of a dusty sky against the clean sky it replaces.
const DUST_LEVEL: f32 = 1.2;

/// Cloud share of the home planet, which the cloud shader was tuned for.
const HOME_CLOUDS: f32 = 0.5;

/// Linear colour of the halo of clean air under a white star.
const HALO_COLOR: Vec3 = Vec3::new(0.32, 0.55, 1.0);

/// The air of a planet, as much of it as the sky needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Air {
    /// Pressure at sea level, the home planet having 1.
    pub pressure: f32,
    /// Tint of the light the air scatters: white leaves the sky as the star alone makes it.
    pub tint: Vec3,
    /// Share of the sky covered by clouds, from 0 to 1.
    pub clouds: f32,
}

impl Air {
    /// The air of the home planet, and of the sky before a world exists.
    pub const HOME: Self = Self {
        pressure: 1.0,
        tint: Vec3::ONE,
        clouds: HOME_CLOUDS,
    };

    /// Reads the air of a planet from its record.
    pub fn of(record: &PlanetRecord) -> Self {
        let look = record.class.look();
        Self {
            pressure: (record.pressure as f32).max(0.0),
            tint: look.air,
            clouds: look.clouds.clamp(0.0, 1.0),
        }
    }

    /// Returns how much of a home sky this air makes at sea level: 0 without air, 1 from the
    /// home pressure up. Standing in thin air is like standing high in the home air.
    pub fn share(&self) -> f32 {
        let full = ((self.pressure - VACUUM) / (1.0 - VACUUM)).clamp(0.0, 1.0);
        1.0 - (1.0 - full) * (1.0 - full)
    }

    /// Returns how many home atmospheres of air the light crosses straight down: 1 up to the
    /// home pressure, more above, levelling off so that no pressure blacks the star out.
    pub fn depth(&self) -> f32 {
        let over = (self.pressure - 1.0).max(0.0);
        1.0 + THICKEN * (1.0 - (-over / THICKEN).exp())
    }

    /// Returns how much closer than at home the haze of this air hides the distance.
    pub fn haze(&self) -> f32 {
        self.depth().sqrt()
    }

    /// Returns the sky colour `base` as this air shows it under a star of colour `star`.
    ///
    /// Clean air leaves it alone. Coloured air is dust: it swaps the blue for its own tint at
    /// the same brightness, the more so the stronger the tint.
    pub fn stain(&self, base: Vec3, star: Vec3) -> Vec3 {
        let pale = self.tint.min_element() / self.tint.max_element().max(1.0e-6);
        let dust = ((1.0 - pale) * DUST).clamp(0.0, 1.0);
        base.lerp(
            self.tint * peak(star) * (luminance(base) * DUST_LEVEL),
            dust,
        )
    }

    /// Returns the cloud cover the shader draws when the cover of the day drifts to `drift` on
    /// the home planet: the same there, none without clouds, more under a heavier sky.
    pub fn cover(&self, drift: f32) -> f32 {
        drift * (self.clouds / HOME_CLOUDS)
    }

    /// Returns the halo of this air seen from space under a star of colour `star`: its linear
    /// colour in xyz and its strength in w, which is 0 for a planet without air.
    pub fn halo(&self, star: Vec3) -> Vec4 {
        self.stain(peak(HALO_COLOR * star), star)
            .extend(self.share() * self.haze())
    }
}
