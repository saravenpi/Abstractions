//! Systems of structure lights: choose which are on near the camera, fade and flicker them, move their flames.

use bevy::light::PointLight;
use bevy::prelude::*;

use super::flame::{Layer, pose};
use super::flicker::{ACTIVE_RANGE, MAX_ACTIVE, fade, flicker, lit_share, pick_active, power};
use super::glow::LightKind;
use super::lights::Glow;
use crate::render::{DayClock, GameCamera, daylight};
use crate::wind::Wind;

/// Seconds between two choices of the active lights.
const CHOOSE_EVERY: f32 = 0.25;

/// Smallest scale of a fading light. Its ray traced emitter shares the scale and must never collapse to a point.
const MIN_SCALE: f32 = 0.05;

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

/// A light with everything its fade changes: the light itself, its size and whether it is shown.
type Lit = (
    &'static mut Glow,
    &'static mut PointLight,
    &'static mut Transform,
    &'static mut Visibility,
);

/// Chooses the nearest lights within range, a few times per second, and lists the nearest fires.
///
/// A chosen light is shown at once and fades in; one that is dropped fades out in `animate` before it is hidden.
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
        if glow.near != on {
            glow.near = on;
        }
        if on {
            vis.set_if_neq(Visibility::Inherited);
        }
    }
    fires.0 = nearest_fires(&spots, &dists);
}

/// Fades the lights toward on or off, flickers them, dims them toward daylight and moves every visible flame shell.
///
/// A light and its flames grow with its level and are hidden only once the level is back to zero.
/// Flames keep their size by day.
pub fn animate(
    time: Res<Time>,
    clock: Res<DayClock>,
    wind: Option<Res<Wind>>,
    mut lights: Query<Lit>,
    mut layers: Query<(&Layer, &mut Transform, &InheritedVisibility), Without<Glow>>,
) {
    let day = daylight(clock.phase);
    let (t, dt) = (time.elapsed_secs(), time.delta_secs());
    for (mut glow, mut light, mut transform, mut vis) in &mut lights {
        if !glow.near && glow.level == 0.0 {
            continue;
        }
        let level = fade(glow.level, glow.near, dt);
        glow.level = level;
        light.intensity = power(glow.kind).0
            * lit_share(glow.kind, day)
            * flicker(glow.kind, t, glow.phase)
            * level;
        transform.scale = Vec3::splat((level * level * (3.0 - 2.0 * level)).max(MIN_SCALE));
        if level == 0.0 {
            *vis = Visibility::Hidden;
        }
    }
    let gust = wind.map_or(Vec3::ZERO, |w| w.velocity());
    for (layer, mut transform, shown) in &mut layers {
        if shown.get() {
            *transform = pose(layer, t, gust);
        }
    }
}
