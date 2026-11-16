//! A fixed pool of smoke puffs that rise from the nearest fires, drift with the wind and shrink away.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::flicker::puff_size;
use super::lights::FireAssets;
use super::lights_sys::Fires;
use crate::wind::Wind;

/// Number of puffs, created once.
const POOL: u32 = 24;

/// One smoke puff of the pool.
#[derive(Component)]
pub struct Puff(u32);

/// Creates the pool of hidden puffs.
pub fn spawn_puffs(mut commands: Commands, assets: Res<FireAssets>) {
    for i in 0..POOL {
        commands.spawn((
            Puff(i),
            Mesh3d(assets.puff.clone()),
            MeshMaterial3d(assets.puff_material.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
}

/// Moves every puff up from its fire, bending with the wind, and hides the pool without fires.
pub fn drift(
    time: Res<Time>,
    wind: Res<Wind>,
    fires: Res<Fires>,
    mut puffs: Query<(&Puff, &mut Transform, &mut Visibility)>,
) {
    let t = time.elapsed_secs();
    for (puff, mut transform, mut vis) in &mut puffs {
        let Some(source) = fires.0.get(puff.0 as usize % fires.0.len().max(1)) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let age = (t * 0.3 + puff.0 as f32 / POOL as f32).fract();
        let sway = Vec3::new(
            (t + puff.0 as f32).sin(),
            0.0,
            (t * 0.7 + puff.0 as f32).cos(),
        ) * 0.15;
        transform.translation =
            *source + Vec3::Y * (0.7 + age * 3.5) + wind.velocity() * age * age * 0.6 + sway;
        transform.scale = Vec3::splat(puff_size(age));
        *vis = Visibility::Inherited;
    }
}
