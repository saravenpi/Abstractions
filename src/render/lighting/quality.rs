//! The shadow settings applied to the lights: the size of the shadow maps, the number of
//! cascades and how far the shadows reach.

use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::tune::{FIRST_BOUND, OVERLAP};
use super::{Moon, Sun};
use crate::settings::{Persistent, Settings, ShadowDetail, ShadowReach};

/// Returns the cascade setup for a shadow `reach` and `detail`: [`ShadowReach::cascades_for`]
/// cascades out to its distance, the first ending at [`FIRST_BOUND`] when there are several and
/// the others spaced by a constant ratio. The ground shader fades the last fifth of the last
/// cascade out, so the end of the reach shows no line.
pub fn cascades(reach: ShadowReach, detail: ShadowDetail) -> CascadeShadowConfig {
    let far = reach.metres();
    CascadeShadowConfigBuilder {
        num_cascades: reach.cascades_for(detail),
        minimum_distance: 0.1,
        first_cascade_far_bound: FIRST_BOUND,
        maximum_distance: far,
        overlap_proportion: OVERLAP,
    }
    .build()
}

/// The shadow cascades of the sun and the moon.
type Cascaded<'w, 's> =
    Query<'w, 's, &'static mut CascadeShadowConfig, Or<(With<Sun>, With<Moon>)>>;

/// Gives the sun and the moon the cascades of the setting and the shadow maps its size, whenever
/// the settings change. A change of the size makes Bevy build the maps again, so it is written
/// only when it differs.
fn apply(
    settings: Res<Persistent<Settings>>,
    mut map: ResMut<DirectionalLightShadowMap>,
    mut lights: Cascaded<'_, '_>,
) {
    let own = &settings.graphics.lighting;
    let size = own.shadow_detail.map_size();
    if map.size != size {
        map.size = size;
    }
    let wanted = cascades(own.shadow_reach, own.shadow_detail);
    for mut config in &mut lights {
        if config.bounds != wanted.bounds {
            *config = wanted.clone();
        }
    }
}

/// Registers the application of the shadow settings, which runs when they change.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply.run_if(resource_changed::<Persistent<Settings>>),
    );
}
