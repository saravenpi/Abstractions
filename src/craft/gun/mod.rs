//! Craftable mineral firearms, ballistics, ADS zoom, and firing state.

pub mod ads;
pub mod ballistics;
pub mod fire;
pub mod mesh;
pub mod spec;
pub mod state;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

pub use spec::{Gun, GunKind, GunStats};
pub use state::{BulletTracer, GunState, MuzzleFlash, ShotFired};

use crate::screens::{in_control, playing};
use crate::view::ViewSet;

/// Registers firearm state, shot events, ADS zoom, firing, and bullet ballistics.
pub fn plugin(app: &mut App) {
    app.init_resource::<GunState>()
        .add_message::<ShotFired>()
        .add_message::<crate::audio::Gunshot>()
        .add_message::<crate::effects::Fling>()
        .add_message::<crate::avatar::strike::Hit>()
        .add_systems(
            Update,
            (
                fire::fire.run_if(in_control).after(ViewSet::Rig),
                ads::decay.run_if(playing),
                ads::ads_fov.after(ViewSet::Rig).run_if(playing),
                ballistics::spawn_tracers
                    .after(fire::fire)
                    .run_if(on_message::<ShotFired>)
                    .run_if(resource_exists::<Assets<Mesh>>)
                    .run_if(resource_exists::<Assets<StandardMaterial>>),
            ),
        )
        .add_systems(
            PostUpdate,
            ballistics::step_tracers
                .after(TransformSystems::Propagate)
                .run_if(playing)
                .run_if(any_with_component::<BulletTracer>)
                .run_if(resource_exists::<avian3d::collider_tree::ColliderTrees>),
        );
}
