//! Run conditions that keep exposure and white-balance easing alive until they converge.

use bevy::camera::Exposure;
use bevy::prelude::*;

use super::balance::{Balanced, D65, adapted};
use super::tune::{STEP, WHITE_STEP};
use super::{Metered, scene_lux, target_ev};
use crate::render::daylight::PlaceLight;
use crate::render::palette::Palette;
use crate::settings::{Persistent, Settings};

/// Returns true while the metered exposure has not yet settled onto its target.
pub(super) fn exposure(
    place: Res<PlaceLight>,
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    metered: Res<Metered>,
) -> bool {
    let live = place.0.air.z > 0.0 && settings.graphics.lighting.auto_exposure;
    let target = match live {
        true => target_ev(scene_lux(&place.0, palette.moon_lux)),
        false => Exposure::EV100_BLENDER,
    };
    (metered.ev - target).abs() > STEP / 2.0
}

/// Returns true while the camera white balance has not yet settled onto its target.
pub(super) fn balance(
    place: Res<PlaceLight>,
    palette: Res<Palette>,
    settings: Res<Persistent<Settings>>,
    balanced: Res<Balanced>,
) -> bool {
    let live = place.0.air.z > 0.0 && settings.graphics.lighting.white_balance;
    let target = match live {
        true => adapted(place.0.noon(), scene_lux(&place.0, palette.moon_lux)),
        false => D65,
    };
    (balanced.white - target).abs().max_element() > WHITE_STEP
}
