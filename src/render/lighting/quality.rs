//! The shadow settings applied to the lights: the size of the shadow maps, the number of
//! cascades and how far the shadows reach.

use bevy::light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use super::tune::{FIRST_SHARE, OVERLAP};
use super::{Moon, Sun};
use crate::settings::{Persistent, Settings, ShadowReach};

/// Returns the cascade setup for a shadow `reach`: two cascades out to its distance, three for the
/// far reach, the first ending at the same share of it. The ground shader fades the last fifth of
/// the last cascade out, so the end of the reach shows no line.
pub fn cascades(reach: ShadowReach) -> CascadeShadowConfig {
    let far = reach.metres();
    CascadeShadowConfigBuilder {
        num_cascades: reach.cascades(),
        minimum_distance: 0.1,
        first_cascade_far_bound: far * FIRST_SHARE,
        maximum_distance: far,
        overlap_proportion: OVERLAP,
    }
    .build()
}

/// Gives the sun and the moon the cascades of the setting and the shadow maps its size, whenever
/// the settings change. A change of the size makes Bevy build the maps again, so it is written
/// only when it differs.
fn apply(
    settings: Res<Persistent<Settings>>,
    mut map: ResMut<DirectionalLightShadowMap>,
    mut lights: Query<&mut CascadeShadowConfig, Or<(With<Sun>, With<Moon>)>>,
) {
    let own = &settings.graphics.lighting;
    let size = own.shadow_detail.map_size();
    if map.size != size {
        map.size = size;
    }
    let wanted = cascades(own.shadow_reach);
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
