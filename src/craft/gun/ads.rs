//! ADS zoom, cooldown decay, recoil spring-damper recovery, and muzzle flash fade.

use bevy::prelude::*;

use super::{GunKind, GunState, MuzzleFlash};
use crate::craft::Pack;
use crate::render::GameCamera;

/// Lifetime in seconds of a spawned muzzle flash light.
pub const FLASH_LIFE: f32 = 0.07;

/// Peak luminous intensity of a spawned muzzle flash light.
pub const FLASH_INTENSITY: f32 = 16_000.0;

/// Fractional FOV reduction when fully aimed down sights with a pistol.
const PISTOL_ZOOM: f32 = 0.22;

/// Fractional FOV reduction when fully aimed down sights with a rifle.
const RIFLE_ZOOM: f32 = 0.38;

/// Returns the maximum FOV zoom reduction factor for firearm `kind`.
pub fn ads_factor(kind: GunKind) -> f32 {
    match kind {
        GunKind::FlintlockPistol => PISTOL_ZOOM,
        GunKind::MineralRifle => RIFLE_ZOOM,
    }
}

/// Returns the perspective FOV multiplier at `ads` blend (`0..=1`) for `kind`.
pub fn ads_fov_scale(ads: f32, kind: GunKind) -> f32 {
    1.0 - ads.clamp(0.0, 1.0) * ads_factor(kind)
}

/// Smoothly interpolates `current` ADS blend toward `1.0` when `aiming`, else `0.0`, over `dt` seconds.
pub fn ads_step(current: f32, aiming: bool, dt: f32) -> f32 {
    let target = if aiming { 1.0 } else { 0.0 };
    let gain = 1.0 - (-14.0 * dt.max(0.0)).exp();
    let next = current + (target - current) * gain;
    if (target - next).abs() < 1.0e-4 {
        target
    } else {
        next
    }
}

/// Steps the recoil spring-damper back toward rest over `dt` seconds.
fn step_recoil(state: &mut GunState, dt: f32) {
    let step = dt.clamp(0.0, 0.05);
    let accel = -140.0 * state.recoil - 22.0 * state.recoil_vel;
    state.recoil_vel += accel * step;
    state.recoil = (state.recoil + state.recoil_vel * step).clamp(-0.15, 1.5);
    if state.recoil.abs() < 1.0e-4 && state.recoil_vel.abs() < 1.0e-4 {
        state.recoil = 0.0;
        state.recoil_vel = 0.0;
    }
}

/// Advances shot cooldown, recoil spring-damper, muzzle flash fade, and ADS decay when holstered.
pub fn decay(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<GunState>,
    pack: Option<Res<Pack>>,
    mut flashes: Query<(Entity, &mut MuzzleFlash, &mut PointLight)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);
    state.flash_age += dt;
    step_recoil(&mut state, dt);
    if pack.as_deref().and_then(Pack::held_gun).is_none() {
        state.ads = (state.ads - dt * 10.0).max(0.0);
        state.reloading = 0.0;
        state.muzzle_world = None;
    }
    for (entity, mut flash, mut light) in &mut flashes {
        flash.age += dt;
        if flash.age >= FLASH_LIFE {
            commands.entity(entity).despawn();
        } else {
            light.intensity = FLASH_INTENSITY * (1.0 - flash.age / FLASH_LIFE).max(0.0);
        }
    }
}

/// Scales the perspective FOV of [`GameCamera`] while aiming down sights.
pub fn ads_fov(
    state: Res<GunState>,
    pack: Option<Res<Pack>>,
    mut cameras: Query<&mut Projection, With<GameCamera>>,
) {
    if state.ads <= 0.001 {
        return;
    }
    let Some(gun) = pack.as_deref().and_then(Pack::held_gun) else {
        return;
    };
    let scale = ads_fov_scale(state.ads, gun.kind);
    for mut proj in &mut cameras {
        if let Projection::Perspective(p) = &mut *proj {
            p.fov *= scale;
        }
    }
}
