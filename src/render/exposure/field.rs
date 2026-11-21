//! The brightness of the lit scene on the screen: what the exposure of the camera makes of the
//! ground around it. A camera set for a sunlit landscape does not register the stars, whatever
//! the sky behind them, as the photographs taken on the Moon show.

use std::f32::consts::PI;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::tune::MEAN_ALBEDO;
use super::{Metered, scene_lux};
use crate::render::daylight::PlaceLight;
use crate::render::palette::Palette;

/// Returns the luminance in linear light on the screen of mean ground lit by `lux` on a camera at
/// the exposure `ev`: the lux times [`MEAN_ALBEDO`] over pi, in cd/m2, times the exposure Bevy
/// takes from an EV100, `2^-ev / 1.2`.
pub fn shown(lux: f32, ev: f32) -> f32 {
    MEAN_ALBEDO * lux.max(0.0) / PI * (-ev).exp2() / 1.2
}

/// The light of the place and the exposure of the camera, for what hides behind the lit scene.
#[derive(SystemParam)]
pub struct Field<'w> {
    /// The light of the planet under the camera.
    place: Option<Res<'w, PlaceLight>>,
    /// The exposure of the camera.
    metered: Option<Res<'w, Metered>>,
    /// The palette, for the light of the moon.
    palette: Res<'w, Palette>,
}

impl Field<'_> {
    /// Returns the luminance on the screen of the mean ground around the camera, 0 away from a
    /// lit planet, as in open space, where nothing around the camera is lit but the ship.
    pub fn luminance(&self) -> f32 {
        let (Some(place), Some(metered)) = (self.place.as_deref(), self.metered.as_deref()) else {
            return 0.0;
        };
        if place.0.air.z <= 0.0 {
            return 0.0;
        }
        shown(scene_lux(&place.0, self.palette.moon_lux), metered.ev)
    }
}
