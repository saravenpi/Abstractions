//! The exposure of the game camera, metered from the light of the place under it.
//!
//! The sun and the moon are in lux and the ambient light in cd/m2, so the camera has an EV100.
//! A fixed EV100 cannot hold a day and a night. This one is metered from the very light the
//! ground shader works with, the place block, and moves slowly toward it, the way an eye adapts,
//! within limits so the night stays dark but readable. Bevy's own `AutoExposure` meters the
//! rendered image with a histogram instead, and the sky, the fog and the clouds are unlit
//! colours it would not scale: here their colours are lifted by the same gain, so every part of
//! the picture moves together.

mod tune;

#[cfg(test)]
mod tests;

use std::f32::consts::PI;

use bevy::camera::Exposure;
use bevy::prelude::*;

use self::tune::{BRIGHTEN_LIMIT, DARKEN_LIMIT, FOLLOW, REFERENCE_LUX, STEP, TO_BRIGHT, TO_DARK};
use super::GameCamera;
use super::daylight::{PlaceLight, PlaceUniform};
use super::palette::Palette;
use super::scatter::luminance;
use crate::settings::{Persistent, Settings};

/// The exposure the camera has now, as an EV100 and as the gain it puts on the picture.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Metered {
    /// Exposure value at ISO 100: a higher value darkens the picture.
    pub ev: f32,
}

impl Default for Metered {
    fn default() -> Self {
        Self {
            ev: Exposure::EV100_BLENDER,
        }
    }
}

impl Metered {
    /// Returns how many times brighter than the reference exposure the picture is: 1 at the
    /// reference, 2 one stop brighter.
    pub fn gain(&self) -> f32 {
        (Exposure::EV100_BLENDER - self.ev).exp2()
    }
}

/// Returns the illuminance in lux that falls on level ground at the camera under `place`: the
/// direct light, the light of the moon `moon_lux`, and the ambient light in cd/m2 times pi. It
/// is 0 without a place, as in space.
pub fn scene_lux(place: &PlaceUniform, moon_lux: f32) -> f32 {
    let sun = luminance(place.eye_sun.truncate());
    let sky = luminance(place.eye_sky.truncate());
    sun + moon_lux.max(0.0) + PI * sky
}

/// Returns the EV100 that meters `lux`: the reference at [`REFERENCE_LUX`], [`FOLLOW`] stops
/// of exposure per stop of light, within [`BRIGHTEN_LIMIT`] and [`DARKEN_LIMIT`] of the reference.
pub fn target_ev(lux: f32) -> f32 {
    let stops = (lux.max(1.0) / REFERENCE_LUX).log2();
    let ev = Exposure::EV100_BLENDER + FOLLOW * stops;
    ev.clamp(
        Exposure::EV100_BLENDER - BRIGHTEN_LIMIT,
        Exposure::EV100_BLENDER + DARKEN_LIMIT,
    )
}

/// Returns `ev` moved toward `target` over `dt` seconds: quickly toward a darker exposure,
/// slowly toward a brighter one.
pub fn eased(ev: f32, target: f32, dt: f32) -> f32 {
    let time = if target > ev { TO_BRIGHT } else { TO_DARK };
    ev + (target - ev) * (1.0 - (-dt.max(0.0) / time).exp())
}

/// Returns the colour `srgb` lifted by `gain` in linear light, clamped to white. The sky, the fog
/// and the clouds are drawn unlit, so they take the gain the lit things get from the camera.
pub fn lifted(srgb: Vec3, gain: f32) -> Vec3 {
    if (gain - 1.0).abs() < 1.0e-4 {
        return srgb;
    }
    let light = Color::srgb(srgb.x, srgb.y, srgb.z).to_linear();
    let lifted = LinearRgba::rgb(light.red * gain, light.green * gain, light.blue * gain);
    let back = Srgba::from(lifted);
    Vec3::new(back.red, back.green, back.blue).min(Vec3::ONE)
}

/// Meters the light of the place and eases the camera exposure toward it. With auto exposure
/// off the camera keeps the reference at once.
fn meter(
    (time, place, palette): (Res<Time>, Res<PlaceLight>, Res<Palette>),
    settings: Res<Persistent<Settings>>,
    mut metered: ResMut<Metered>,
    cameras: Query<(Entity, Option<&Exposure>), With<GameCamera>>,
    mut commands: Commands,
) {
    let live = place.0.air.z > 0.0 && settings.graphics.lighting.auto_exposure;
    let target = match live {
        true => target_ev(scene_lux(&place.0, palette.moon_lux)),
        false => Exposure::EV100_BLENDER,
    };
    let ev = match live {
        true => eased(metered.ev, target, time.delta_secs()),
        false => target,
    };
    metered.set_if_neq(Metered { ev });
    for (camera, exposure) in &cameras {
        if exposure.is_none_or(|own| (own.ev100 - metered.ev).abs() > STEP / 2.0) {
            commands
                .entity(camera)
                .insert(Exposure { ev100: metered.ev });
        }
    }
}

/// Registers the metered exposure, which runs when the light of the place or the settings change.
pub(super) fn plugin(app: &mut App) {
    let changed = resource_changed::<PlaceLight>.or_else(resource_changed::<Persistent<Settings>>);
    app.init_resource::<Metered>()
        .add_systems(PostUpdate, meter.run_if(changed).after(super::PlaceLit));
}
