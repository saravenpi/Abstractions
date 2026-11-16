//! Systems of structure lights: choose which are on near the camera, flicker them and dim them by day.

use bevy::light::PointLight;
use bevy::prelude::*;

use super::flicker::{ACTIVE_RANGE, MAX_ACTIVE, flicker, night_share, pick_active, power};
use super::glow::LightKind;
use super::lights::Glow;
use crate::render::{DayClock, GameCamera, daylight};

/// Seconds between two choices of the active lights.
const CHOOSE_EVERY: f32 = 0.25;

/// Fires that smoke rises from: the nearest few campfires and braziers, in world space.
#[derive(Resource, Default)]
pub struct Fires(pub Vec<Vec3>);

/// Returns the positions of the four nearest campfires and braziers within range.
fn nearest_fires(spots: &[(Vec3, LightKind)], dists: &[f32]) -> Vec<Vec3> {
    let mut burning: Vec<(f32, Vec3)> = spots
        .iter()
        .zip(dists)
        .filter(|((_, k), d)| {
            matches!(k, LightKind::Campfire | LightKind::Brazier) && **d < ACTIVE_RANGE
        })
        .map(|((p, _), d)| (*d, *p))
        .collect();
    burning.sort_by(|a, b| a.0.total_cmp(&b.0));
    burning.into_iter().take(4).map(|(_, p)| p).collect()
}

/// Switches on the nearest lights within range, a few times per second, and lists the nearest fires.
pub fn select(
    time: Res<Time>,
    mut since: Local<f32>,
    camera: Query<&GlobalTransform, With<GameCamera>>,
    mut glows: Query<(&GlobalTransform, &mut Glow, &mut Visibility)>,
    mut fires: ResMut<Fires>,
) {
    *since += time.delta_secs();
    let Ok(eye) = camera.single().map(|c| c.translation()) else {
        return;
    };
    if *since < CHOOSE_EVERY {
        return;
    }
    *since = 0.0;
    let spots: Vec<(Vec3, LightKind)> = glows
        .iter()
        .map(|(t, g, _)| (t.translation(), g.kind))
        .collect();
    let dists: Vec<f32> = spots.iter().map(|(p, _)| p.distance(eye)).collect();
    let on = pick_active(&dists, MAX_ACTIVE, ACTIVE_RANGE);
    for ((_, mut glow, mut vis), on) in glows.iter_mut().zip(on) {
        glow.near = on;
        *vis = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    fires.0 = nearest_fires(&spots, &dists);
}

/// Flickers the lit lights and dims them toward daylight; flames keep their size by day.
pub fn animate(
    time: Res<Time>,
    clock: Res<DayClock>,
    mut lights: Query<(&Glow, &mut PointLight, &mut Transform)>,
) {
    let night = night_share(daylight(clock.phase));
    let t = time.elapsed_secs();
    for (glow, mut light, mut transform) in &mut lights {
        if !glow.near {
            continue;
        }
        let steady = glow.kind == LightKind::Lamp;
        let f = if steady { 1.0 } else { flicker(t, glow.phase) };
        light.intensity = power(glow.kind).0 * night * f;
        transform.scale = if steady {
            Vec3::ONE
        } else {
            Vec3::new(1.0, f * 1.15, 1.0)
        };
    }
}
