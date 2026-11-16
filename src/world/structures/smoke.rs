//! Fixed pools of smoke puffs and embers that rise from the nearest fires and drift with the wind.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::lights::FireAssets;
use super::lights_sys::Fires;
use crate::wind::Wind;

/// Number of smoke puffs, created once.
const POOL: u32 = 24;

/// Number of embers, created once.
const EMBERS: u32 = 16;

/// Diameter of an ember in meters.
const EMBER_SIZE: f32 = 0.035;

/// One smoke puff of the pool.
#[derive(Component)]
pub struct Puff(u32);

/// One ember of the pool.
#[derive(Component)]
pub struct Ember(u32);

/// Transform, visibility and material of a smoke puff.
type Puffs<'w, 's> = Query<
    'w,
    's,
    (
        &'static Puff,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
>;

/// Transform and visibility of an ember.
type Embers<'w, 's> = Query<
    'w,
    's,
    (
        &'static Ember,
        &'static mut Transform,
        &'static mut Visibility,
    ),
    Without<Puff>,
>;

/// Returns the size of a smoke puff at `age` in `[0, 1)`: it swells, then shrinks away.
pub fn puff_size(age: f32) -> f32 {
    match age < 0.75 {
        true => 0.12 + age / 0.75 * 0.4,
        false => 0.52 * (1.0 - (age - 0.75) / 0.25).max(0.0),
    }
}

/// Returns which smoke material a puff of `age` in `[0, 1)` wears: dark soot first, pale grey last.
pub fn puff_shade(age: f32) -> usize {
    ((age.clamp(0.0, 1.0) * 3.0) as usize).min(2)
}

/// Returns the offset of an ember from its fire at `age` in `[0, 1)`: a fast wobbling rise bent by `wind`.
pub fn ember_path(age: f32, slot: u32, wind: Vec3) -> Vec3 {
    let turn = slot as f32 * 2.4;
    let spread = 0.1 + 0.25 * age;
    let wobble = Vec3::new(
        (turn + age * 9.0).sin(),
        0.0,
        (turn * 1.3 + age * 7.0).cos(),
    ) * spread;
    wobble + Vec3::Y * (0.1 + age * 2.2) + wind * age * age * 0.35
}

/// Creates the pools of hidden puffs and embers.
pub fn spawn_puffs(mut commands: Commands, assets: Res<FireAssets>) {
    let hidden = || (Transform::default(), Visibility::Hidden, NotShadowCaster);
    for i in 0..POOL {
        let material = MeshMaterial3d(assets.smoke[0].clone());
        commands.spawn((Puff(i), Mesh3d(assets.puff.clone()), material, hidden()));
    }
    for i in 0..EMBERS {
        let material = MeshMaterial3d(assets.ember.clone());
        commands.spawn((Ember(i), Mesh3d(assets.puff.clone()), material, hidden()));
    }
}

/// Moves every ember up from its fire: it shrinks as it cools and dies before the smoke thins.
fn glint(embers: &mut Embers, fires: &[Vec3], t: f32, wind: Vec3) {
    for (ember, mut transform, mut vis) in embers {
        let Some(source) = fires.get(ember.0 as usize % fires.len().max(1)) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        let age = (t * (0.5 + 0.05 * (ember.0 % 5) as f32) + ember.0 as f32 * 0.37).fract();
        transform.translation = *source + ember_path(age, ember.0, wind);
        transform.scale = Vec3::splat(EMBER_SIZE * 0.5 * (1.0 - age * age));
        vis.set_if_neq(Visibility::Inherited);
    }
}

/// Moves every puff and ember up from its fire, bending with the wind, and hides the pools without fires.
///
/// A puff darkens to soot near the fire and pales as it ages.
pub fn drift(
    time: Res<Time>,
    wind: Res<Wind>,
    fires: Res<Fires>,
    assets: Res<FireAssets>,
    mut pools: (Puffs, Embers),
) {
    let t = time.elapsed_secs();
    for (puff, mut transform, mut vis, mut material) in &mut pools.0 {
        let Some(source) = fires.0.get(puff.0 as usize % fires.0.len().max(1)) else {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        };
        let age = (t * 0.3 + puff.0 as f32 / POOL as f32).fract();
        let turn = puff.0 as f32;
        let sway = Vec3::new((t + turn).sin(), 0.0, (t * 0.7 + turn).cos()) * 0.15;
        transform.translation =
            *source + Vec3::Y * (0.7 + age * 3.5) + wind.velocity() * age * age * 0.6 + sway;
        transform.scale = Vec3::splat(puff_size(age));
        material.set_if_neq(MeshMaterial3d(assets.smoke[puff_shade(age)].clone()));
        vis.set_if_neq(Visibility::Inherited);
    }
    glint(&mut pools.1, &fires.0, t, wind.velocity());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Puffs swell then vanish and pale with age; embers start at the fire, always rise and stay close to it.
    #[test]
    fn puffs_and_embers_rise_and_fade() {
        assert!(puff_size(0.999) < 0.01 && puff_size(0.7) > puff_size(0.0));
        assert_eq!(
            [puff_shade(0.0), puff_shade(0.5), puff_shade(0.999)],
            [0, 1, 2]
        );
        for slot in 0..EMBERS {
            let path: Vec<Vec3> = (0..=20)
                .map(|i| ember_path(i as f32 / 20.0, slot, Vec3::ZERO))
                .collect();
            assert!(
                path.windows(2).all(|w| w[1].y > w[0].y),
                "ember {slot} falls"
            );
            assert!(path[0].length() < 0.3 && path.iter().all(|p| p.xz().length() < 0.6));
        }
    }
}
