//! Lighting the bodies each frame: every planet and moon from its star through the shadows of
//! its family, the storm of a gas giant turning with it, and the rings from their star.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Body;
use super::giant::Giant;
use super::material::BodyMaterial;
use super::rings::RingMaterial;
use crate::render::vantage::Scene;
use crate::world::spin;

/// Lights every planet and moon from the direction of its star, dimmed where another body of
/// its planet hides the star, and turns its spin axis and the storm of a giant into render axes.
pub(super) fn planets(
    scene: Scene,
    planets: Query<(&Body, &MeshMaterial3d<BodyMaterial>)>,
    mut materials: ResMut<Assets<BodyMaterial>>,
) {
    let Some(near) = scene.local.0.as_ref() else {
        return;
    };
    let (to_render, time) = (scene.vantage.observer.to_render, scene.clock.seconds);
    for (body, handle) in &planets {
        let Body::Planet(id) = *body else {
            continue;
        };
        let Some((planet, pos)) = scene.body(id) else {
            continue;
        };
        let Some(mut material) = materials.get_mut(&handle.0) else {
            continue;
        };
        let uniform = &mut material.body;
        let lit = near.eclipse(pos, id, time);
        uniform.light = scene.vantage.sunward(pos).extend(lit);
        uniform.axis = (to_render * DVec3::Y).as_vec3().extend(uniform.axis.w);
        if let Some(giant) = Giant::of(&planet.record, planet.seed) {
            let spot = to_render * (spin(planet.day_seconds, time) * giant.spot);
            uniform.storm = spot.as_vec3().extend(giant.size);
        }
    }
}

/// Lights the rings of every ringed planet from the direction of its star.
pub(super) fn rings(
    scene: Scene,
    rings: Query<(&Body, &MeshMaterial3d<RingMaterial>)>,
    mut materials: ResMut<Assets<RingMaterial>>,
) {
    for (body, handle) in &rings {
        let Body::Rings(id) = *body else {
            continue;
        };
        let Some((_, pos)) = scene.body(id) else {
            continue;
        };
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.light(pos);
        }
    }
}
