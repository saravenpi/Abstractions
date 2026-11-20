//! Shading that depends on how far and how high the camera is: the ambient occlusion pass is
//! taken off the camera when it is too high above the ground to darken anything near.
//!
//! The ground shader does the rest, in `relit` of `daylight.wesl`: it fades the shadows out over
//! the last fifth of the last shadow cascade and the occlusion out with the distance, so no
//! line shows where the shadow maps end and no halo shows at the seams of far chunks.

mod tune;

#[cfg(test)]
mod tests;

use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;

use self::tune::{OCCLUSION_CEILING, OCCLUSION_MARGIN};
use super::GameCamera;
use super::fog::Clearance;
use crate::settings::{Persistent, Settings};

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

/// Gives the camera the ambient occlusion pass near the ground and takes it off high above.
fn occlusion(
    mut commands: Commands,
    aloft: Res<Clearance>,
    settings: Res<Persistent<Settings>>,
    cameras: Query<(Entity, Has<ScreenSpaceAmbientOcclusion>), With<GameCamera>>,
) {
    let allowed = settings.graphics.ambient_occlusion;
    for (camera, has) in &cameras {
        let wanted = occluding(allowed, aloft.0, has);
        if wanted && !has {
            commands
                .entity(camera)
                .insert(ScreenSpaceAmbientOcclusion::default());
        } else if !wanted && has {
            commands
                .entity(camera)
                .remove::<ScreenSpaceAmbientOcclusion>();
        }
    }
}

/// Registers the occlusion pass control, which runs when the camera height or the settings change.
pub(super) fn plugin(app: &mut App) {
    let changed = resource_changed::<Clearance>.or_else(resource_changed::<Persistent<Settings>>);
    app.add_systems(Update, occlusion.run_if(changed));
}
