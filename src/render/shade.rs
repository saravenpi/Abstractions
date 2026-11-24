//! Shading that depends on how far and how high the camera is: the ambient occlusion pass is
//! taken off the camera when it is too high above the ground to darken anything near.
//!
//! The ground shader does the rest, in `relit` of `daylight.wesl`: it fades the shadows out over
//! the last fifth of the last shadow cascade and the occlusion out with the distance, so no
//! line shows where the shadow maps end and no halo shows at the seams of far chunks.

mod tune;

#[cfg(test)]
mod tests;

use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;

use self::tune::{OCCLUSION_CEILING, OCCLUSION_MARGIN};
use super::GameCamera;
use super::fog::Clearance;
use super::murk::Waterline;
use crate::settings::{Persistent, Settings};

/// Metres above the water surface within which the depth prepass stays ready for [`super::murk`].
const WATER_DEPTH_MARGIN: f32 = 2.0;

/// Active prepass markers on the game camera.
type Passes = (Has<ScreenSpaceAmbientOcclusion>, Has<DepthPrepass>);

/// Returns whether the camera carries the ambient occlusion pass at `clearance` metres above the
/// ground, when the setting allows it and the camera `has` it now. A camera that has it keeps it
/// [`OCCLUSION_MARGIN`] metres higher than one that does not takes it on.
fn occluding(allowed: bool, clearance: f32, has: bool) -> bool {
    let ceiling = match has {
        true => OCCLUSION_CEILING + OCCLUSION_MARGIN,
        false => OCCLUSION_CEILING,
    };
    allowed && clearance < ceiling
}

/// Keeps ambient occlusion and paired depth/normal prepasses synchronized on `camera`.
fn sync(commands: &mut Commands, camera: Entity, have: (bool, bool), want: (bool, bool)) {
    let mut entity = commands.entity(camera);
    if !want.0 && have.0 {
        entity.remove::<ScreenSpaceAmbientOcclusion>();
    }
    if want.1 != have.1 {
        if want.1 {
            entity.insert((DepthPrepass, NormalPrepass));
        } else {
            entity.remove::<(DepthPrepass, NormalPrepass)>();
        }
    }
    if want.0 && !have.0 {
        entity.insert(ScreenSpaceAmbientOcclusion::default());
    }
}

/// Gives the camera ambient occlusion near the ground and paired depth/normal prepasses only when
/// needed near liquid surfaces or for occlusion, avoiding un-dithered depth prepass artifacts.
fn occlusion(
    mut commands: Commands,
    aloft: Res<Clearance>,
    settings: Res<Persistent<Settings>>,
    waterline: Option<Res<Waterline>>,
    cameras: Query<(Entity, Passes), With<GameCamera>>,
) {
    let allowed = settings.graphics.ambient_occlusion;
    let water = waterline
        .as_deref()
        .is_some_and(|w| w.sea && w.height < WATER_DEPTH_MARGIN);
    for (camera, (has_ao, has_depth)) in &cameras {
        let want_ao = occluding(allowed, aloft.0, has_ao);
        sync(
            &mut commands,
            camera,
            (has_ao, has_depth),
            (want_ao, want_ao || water),
        );
    }
}

/// Registers the prepass control, which runs when camera clearance, waterline, or settings change.
pub(super) fn plugin(app: &mut App) {
    let water = resource_exists::<Waterline>.and_then(resource_changed::<Waterline>);
    let changed = resource_changed::<Clearance>
        .or_else(resource_changed::<Persistent<Settings>>)
        .or_else(water);
    app.add_systems(Update, occlusion.run_if(changed));
}
